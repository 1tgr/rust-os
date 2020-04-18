use crate::client::pipe::ClientPipe;
use crate::client::portal::{ClientPortalId, ClientPortalSystem};
use crate::components::OnInput;
use crate::system::System;
use crate::types::Event;
use crate::Result;
use alloc::rc::Rc;
use core::cell::RefCell;
use hecs::World;

pub struct App {
    pipe: Rc<RefCell<ClientPipe>>,
    world: World,
    systems: Vec<Box<dyn System>>,
}

impl App {
    pub fn new() -> Self {
        let pipe = Rc::new(RefCell::new(ClientPipe::new()));
        let mut systems: Vec<Box<dyn System>> = Vec::new();
        systems.push(Box::new(ClientPortalSystem::new(pipe.clone())));
        Self {
            pipe,
            world: World::new(),
            systems,
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn checkpoint(&mut self) -> Result<usize> {
        self.pipe.borrow_mut().checkpoint()
    }

    pub fn wait_for_event(&mut self) -> Result<Event> {
        for system in self.systems.iter_mut() {
            system.run(&mut self.world)?;
        }

        self.pipe.borrow_mut().wait_for_event()
    }

    pub fn dispatch_event(&mut self, event: &Event) -> Result<()> {
        let mut inputs = Vec::new();

        for (entity, (&ClientPortalId(id), &OnInput(ref on_input))) in
            self.world.query::<(&ClientPortalId, &OnInput)>().iter()
        {
            match *event {
                Event::Input { portal_id, ref input } if portal_id == id => {
                    inputs.push((entity, on_input.clone(), input));
                }
                _ => (),
            }
        }

        for (entity, on_input, input) in inputs {
            (on_input)(&mut self.world, entity, input)?;
        }

        Ok(())
    }
}
