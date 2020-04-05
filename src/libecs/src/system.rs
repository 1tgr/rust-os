use crate::compat::Result;
use crate::component::ComponentStorage;

pub trait System {
    fn run(&mut self, e: &mut ComponentStorage) -> Result<()>;
}
