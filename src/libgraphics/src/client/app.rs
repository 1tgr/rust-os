use crate::client::portal::ClientPortalSystem;
use crate::system::System;
use crate::types::Event;
use crate::Result;
use hecs::World;

pub struct App {
    world: World,
    system: ClientPortalSystem,
}

impl App {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            system: ClientPortalSystem::new(),
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
        self.system.run(&mut self.world)?;
        self.system.pipe.wait_for_event()
    }

    pub fn dispatch_event(&mut self, event: Event) -> Result<()> {
        self.system.dispatch_event(&mut self.world, event)
    }
}
