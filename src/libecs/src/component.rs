use crate::archetype::{Archetype, ArchetypeStorage};
use crate::compat::{ErrNum, Result};
use crate::entity::{Entity, EntityInner};
use crate::system::System;
use alloc::collections::btree_map::{BTreeMap, Entry};
use alloc::collections::btree_set::BTreeSet;
use alloc::rc::Rc;
use core::any::TypeId;
use core::borrow;
use core::cell::RefCell;
use core::mem;

fn get2_mut<'a, K, V, Q, R>(map: &'a mut BTreeMap<K, V>, key1: &'a Q, key2: &'a R) -> Option<(&'a mut V, &'a mut V)>
where
    K: borrow::Borrow<Q> + borrow::Borrow<R> + Ord,
    Q: Ord,
    R: Ord,
{
    let value1 = map.get_mut(key1)? as *mut V;
    let value2 = map.get_mut(key2)?;
    assert_ne!(value1, value2 as *mut V);
    Some((unsafe { &mut *value1 }, value2))
}

pub struct ComponentStorage {
    archetypes: BTreeMap<Rc<Archetype>, ArchetypeStorage>,
    deleted_archetypes: BTreeMap<Rc<Archetype>, ArchetypeStorage>,
    systems: Vec<Rc<RefCell<dyn System>>>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            archetypes: BTreeMap::new(),
            deleted_archetypes: BTreeMap::new(),
            systems: Vec::new(),
        }
    }

    pub fn add_component<C>(&mut self, entity: &Entity, state: C)
    where
        C: 'static,
    {
        let entity = &mut *entity.inner.borrow_mut();
        if let Some(EntityInner { archetype, index }) = entity {
            let prev_archetype = Rc::clone(archetype);
            Rc::make_mut(archetype).insert(TypeId::of::<C>());

            self.archetypes
                .entry(Rc::clone(archetype))
                .or_insert_with(|| ArchetypeStorage::new());

            let (prev_storage, storage) = get2_mut(&mut self.archetypes, &prev_archetype, archetype).unwrap();
            *index = ArchetypeStorage::move_entity(prev_storage, storage, *index, state);
        } else {
            let mut archetype = BTreeSet::new();
            archetype.insert(TypeId::of::<C>());

            let archetype = Rc::new(archetype);

            let index = self
                .archetypes
                .entry(archetype.clone())
                .or_insert_with(|| ArchetypeStorage::new())
                .add_entity(state);

            *entity = Some(EntityInner { archetype, index });
        }
    }

    pub fn add_system<S>(&mut self, system: S)
    where
        S: System + 'static,
    {
        self.systems.push(Rc::new(RefCell::new(system)));
    }

    pub fn remove_component<C>(&mut self, entity: &Entity) -> bool
    where
        C: 'static,
    {
        let entity = &mut *entity.inner.borrow_mut();
        let EntityInner { archetype, index } = if let Some(inner) = entity {
            inner
        } else {
            return false;
        };

        if !archetype.contains(&TypeId::of::<C>()) {
            return false;
        }

        let mut entry = if let Entry::Occupied(entry) = self.archetypes.entry(Rc::clone(archetype)) {
            entry
        } else {
            panic!();
        };

        let prev_storage = entry.get_mut();
        let prev_archetype = Rc::clone(archetype);
        Rc::make_mut(archetype).remove(&TypeId::of::<C>());

        let storage = self
            .deleted_archetypes
            .entry(prev_archetype)
            .or_insert_with(|| ArchetypeStorage::new());

        *index = ArchetypeStorage::move_entity(prev_storage, storage, *index, ());

        if prev_storage.is_empty() {
            entry.remove();
        }

        if archetype.is_empty() {
            *entity = None;
        }

        true
    }

    pub fn clear_entity(&mut self, entity: &Entity) -> bool {
        let entity = &mut *entity.inner.borrow_mut();
        let EntityInner { archetype, index } = if let Some(inner) = mem::replace(entity, None) {
            inner
        } else {
            return false;
        };

        if archetype.is_empty() {
            return false;
        }

        let mut entry = if let Entry::Occupied(entry) = self.archetypes.entry(archetype.clone()) {
            entry
        } else {
            panic!();
        };

        let prev_storage = entry.get_mut();

        let storage = self
            .deleted_archetypes
            .entry(archetype.clone())
            .or_insert_with(|| ArchetypeStorage::new());

        ArchetypeStorage::move_entity(prev_storage, storage, index, ());

        if prev_storage.is_empty() {
            entry.remove();
        }

        true
    }

    pub fn component<C>(&self, entity: &Entity) -> Option<&C>
    where
        C: 'static,
    {
        if let Some(EntityInner { ref archetype, index }) = *entity.inner.borrow() {
            self.archetypes.get(archetype)?.as_slice().get(index)
        } else {
            None
        }
    }

    pub fn update_component<C, F, T>(&mut self, entity: &Entity, f: F) -> Result<T>
    where
        C: 'static,
        F: FnOnce(&mut C) -> Result<T>,
    {
        if let Some(EntityInner { ref archetype, index }) = *entity.inner.borrow() {
            if let Some(storage) = self.archetypes.get_mut(archetype) {
                if let Some(state) = storage.as_mut_slice().get_mut(index) {
                    return f(state);
                }
            }
        }

        Err(ErrNum::InvalidArgument)
    }

    pub fn components<C>(&self) -> impl Iterator<Item = &C>
    where
        C: 'static,
    {
        self.archetypes.values().flat_map(|storage| storage.as_slice().iter())
    }

    pub fn components_mut<C>(&mut self) -> impl Iterator<Item = &mut C>
    where
        C: 'static,
    {
        self.archetypes
            .values_mut()
            .flat_map(|storage| storage.as_mut_slice().iter_mut())
    }

    pub fn deleted_components<C>(&self) -> impl Iterator<Item = &C>
    where
        C: 'static,
    {
        self.deleted_archetypes
            .values()
            .flat_map(|storage| storage.as_slice().iter())
    }

    pub fn run_systems(&mut self) -> Result<()> {
        for system in self.systems.clone() {
            system.borrow_mut().run(self)?;
        }

        self.deleted_archetypes.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::component::ComponentStorage;
    use crate::entity::Entity;

    macro_rules! assert_iter {
        ($left:expr, $($right:expr),*) => {
            assert_eq!($left.cloned().collect::<Vec<_>>(), vec![$($right,)*])
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Component1(pub u32);

    #[test]
    fn test_one_component() {
        let empty_entity = Entity::new();
        let mut storage = ComponentStorage::new();
        let entity1 = Entity::new();
        assert_eq!(entity1, empty_entity);

        storage.add_component(&entity1, Component1(1));
        assert_ne!(entity1, empty_entity);
        assert_iter!(storage.components::<Component1>(), Component1(1));

        for Component1(n) in storage.components_mut() {
            *n += 10;
        }

        assert_iter!(storage.components::<Component1>(), Component1(11));

        let entity2 = Entity::new();
        storage.add_component(&entity2, Component1(2));
        assert_iter!(storage.components::<Component1>(), Component1(11), Component1(2));

        for Component1(n) in storage.components_mut() {
            *n += 10;
        }

        assert_iter!(storage.components::<Component1>(), Component1(21), Component1(12));

        storage.remove_component::<Component1>(&entity2);
        assert_eq!(entity2, empty_entity);
        assert_iter!(storage.components::<Component1>(), Component1(21));

        storage.clear_entity(&entity1);
        assert_eq!(entity1, empty_entity);
        assert_eq!(storage.components::<Component1>().next(), None);
    }
}
