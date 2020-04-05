use crate::client::portal::ClientPortalSystem;
use crate::client::ClientPipe;
use crate::types::Event;
use crate::ClientPortal;
use ecs::{ComponentStorage, Entity};
use os::Result;

pub struct App {
    pipe: Entity,
    entities: ComponentStorage,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut entities = ComponentStorage::new();
        let pipe = Entity::new();
        entities.add_component(&pipe, ClientPipe::new());
        entities.add_system(ClientPortalSystem);
        Ok(Self { pipe, entities })
    }

    pub fn entities(&self) -> &ComponentStorage {
        &self.entities
    }

    pub fn entities_mut(&mut self) -> &mut ComponentStorage {
        &mut self.entities
    }

    pub fn checkpoint(&mut self) -> Result<usize> {
        self.entities
            .update_component(&self.pipe, |state: &mut ClientPipe| state.checkpoint())
    }

    pub fn wait_for_event(&mut self) -> Result<Event> {
        self.entities.run_systems()?;
        self.entities
            .update_component(&self.pipe, |state: &mut ClientPipe| state.wait_for_event())
    }

    pub fn dispatch_event(&mut self, event: &Event) {
        for portal in self.entities.components_mut::<ClientPortal>() {
            portal.dispatch_event(event);
        }
    }
}
