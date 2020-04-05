use crate::archetype::Archetype;
use alloc::rc::Rc;
use core::cell::RefCell;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct EntityInner {
    pub archetype: Rc<Archetype>,
    pub index: usize,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Entity {
    pub(crate) inner: Rc<RefCell<Option<EntityInner>>>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(None)),
        }
    }
}
