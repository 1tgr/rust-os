use crate::client::portal::ClientPortalSystem;
use crate::system::System;
use crate::types::Event;
use crate::widgets;
use crate::Result;
use hecs::World;

pub struct App {
    world: World,
    system: ClientPortalSystem,
    systems: Vec<Box<dyn System>>,
}

impl App {
    pub fn new() -> Self {
        let mut systems: Vec<Box<dyn System>> = Vec::new();
        widgets::register(&mut systems);

        Self {
            world: World::new(),
            system: ClientPortalSystem::new(),
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
        self.system.pipe.checkpoint()
    }

    pub fn wait_for_event(&mut self) -> Result<Event> {
        for system in self.systems.iter_mut() {
            system.run(&mut self.world)?;
        }

        self.system.run(&mut self.world)?;
        self.system.pipe.wait_for_event()
    }

    pub fn dispatch_event(&mut self, event: Event) -> Result<()> {
        self.system.dispatch_event(&mut self.world, event)
    }
}
