use alloc::collections::btree_map::BTreeMap;
use core::any::{Any, TypeId};

pub struct TypeMap {
    pub data: BTreeMap<TypeId, Box<dyn Any>>,
}

impl TypeMap {
    pub fn new() -> Self {
        Self { data: BTreeMap::new() }
    }

    pub fn get<K, V>(&self) -> Option<&V>
    where
        K: 'static,
        V: 'static,
    {
        Some(self.data.get(&TypeId::of::<K>())?.downcast_ref::<V>().unwrap())
    }

    pub fn get_mut<K, V>(&mut self) -> Option<&mut V>
    where
        K: 'static,
        V: 'static,
    {
        Some(self.data.get_mut(&TypeId::of::<K>())?.downcast_mut::<V>().unwrap())
    }

    pub fn get_or_insert_with<K, V, F>(&mut self, f: F) -> &mut V
    where
        K: 'static,
        V: 'static,
        F: FnOnce() -> V,
    {
        self.data
            .entry(TypeId::of::<K>())
            .or_insert_with(|| Box::new(f()))
            .downcast_mut::<V>()
            .unwrap()
    }
}
