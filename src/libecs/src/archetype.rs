use crate::type_map::TypeMap;
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::btree_set::BTreeSet;
use alloc::rc::Rc;
use core::any::{Any, TypeId};

#[derive(Clone)]
struct ArchetypeStorageAccessor {
    empty_vec: Rc<dyn Fn() -> Box<dyn Any>>,
    move_entity: Rc<dyn Fn(&mut Box<dyn Any>, &mut Box<dyn Any>, usize)>,
    remove_entity: Rc<dyn Fn(&mut Box<dyn Any>, usize)>,
}

impl ArchetypeStorageAccessor {
    pub fn new<C, EmptyVec, MoveEntity, RemoveEntity>(
        empty_vec: EmptyVec,
        move_entity: MoveEntity,
        remove_entity: RemoveEntity,
    ) -> Self
    where
        C: 'static,
        EmptyVec: Fn() -> Vec<C> + 'static,
        MoveEntity: Fn(&mut Vec<C>, &mut Vec<C>, usize) + 'static,
        RemoveEntity: Fn(&mut Vec<C>, usize) + 'static,
    {
        Self {
            empty_vec: Rc::new(move || Box::new(empty_vec())),

            move_entity: Rc::new(move |from_dyn, to_dyn, from_index| {
                move_entity(
                    from_dyn.downcast_mut().unwrap(),
                    to_dyn.downcast_mut().unwrap(),
                    from_index,
                )
            }),

            remove_entity: Rc::new(move |vec_dyn, index| remove_entity(vec_dyn.downcast_mut().unwrap(), index)),
        }
    }
}

pub type Archetype = BTreeSet<TypeId>;

pub struct ArchetypeStorage {
    components: TypeMap,
    /* {Component}, Vec<{Component}> */
    accessors: BTreeMap<TypeId, ArchetypeStorageAccessor>,
    next_index: usize,
}

impl ArchetypeStorage {
    pub fn new() -> Self {
        Self {
            components: TypeMap::new(),
            accessors: BTreeMap::new(),
            next_index: 0,
        }
    }

    fn ensure<C>(&mut self) -> &mut Vec<C>
    where
        C: 'static,
    {
        self.accessors.entry(TypeId::of::<C>()).or_insert_with(|| {
            ArchetypeStorageAccessor::new(
                || Vec::<C>::new(),
                |from, to, from_index| {
                    let value = from.remove(from_index);
                    to.push(value);
                },
                |vec, index| {
                    vec.remove(index);
                },
            )
        });

        self.components.get_or_insert_with::<C, _, _>(|| Vec::new())
    }

    fn get<C>(&self) -> Option<&Vec<C>>
    where
        C: 'static,
    {
        self.components.get::<C, _>()
    }

    fn get_mut<C>(&mut self) -> Option<&mut Vec<C>>
    where
        C: 'static,
    {
        self.components.get_mut::<C, _>()
    }

    pub fn add_entity<C>(&mut self, state: C) -> usize
    where
        C: 'static,
    {
        let index = self.next_index;
        self.next_index += 1;
        self.ensure().push(state);
        index
    }

    pub fn move_entity<C>(from: &mut Self, to: &mut Self, from_index: usize, state: C) -> usize
    where
        C: 'static,
    {
        assert_eq!(from_index, from.next_index - 1);
        from.next_index -= 1;

        for (&type_id, from_dyn) in from.components.data.iter_mut() {
            let accessor = from.accessors.get(&type_id).unwrap();
            to.accessors.entry(type_id).or_insert_with(|| accessor.clone());

            let to_dyn = to
                .components
                .data
                .entry(type_id)
                .or_insert_with(|| (accessor.empty_vec)());

            (accessor.move_entity)(from_dyn, to_dyn, from_index);
        }

        let index = to.next_index;
        to.next_index += 1;

        let to_vec = to.ensure();
        assert_eq!(to_vec.len(), index);
        to_vec.push(state);

        index
    }

    pub fn is_empty(&self) -> bool {
        self.next_index == 0
    }

    pub fn as_slice<C>(&self) -> &[C]
    where
        C: 'static,
    {
        self.get().map(|v| &v[..]).unwrap_or(&[])
    }

    pub fn as_mut_slice<C>(&mut self) -> &mut [C]
    where
        C: 'static,
    {
        self.get_mut().map(|v| &mut v[..]).unwrap_or(&mut [])
    }
}
