use crate::Result;
use core::mem;
use hashbrown::hash_map::Entry;
use hashbrown::HashMap;
use hecs::{Entity, World};

pub trait System {
    fn run(&mut self, world: &mut World) -> Result<()>;
}

pub trait Hmm {
    type Owned;

    fn into_owned(self) -> Self::Owned;
}

impl<'a, T> Hmm for &'a T
where
    T: Clone,
{
    type Owned = T;

    fn into_owned(self) -> T {
        self.clone()
    }
}

impl Hmm for () {
    type Owned = ();

    fn into_owned(self) -> Self::Owned {
        ()
    }
}

impl<A> Hmm for (A,)
where
    A: Hmm,
{
    type Owned = (A::Owned,);

    fn into_owned(self) -> Self::Owned {
        (self.0.into_owned(),)
    }
}

impl<A, B> Hmm for (A, B)
where
    A: Hmm,
    B: Hmm,
{
    type Owned = (A::Owned, B::Owned);

    fn into_owned(self) -> Self::Owned {
        (self.0.into_owned(), self.1.into_owned())
    }
}

pub struct DeletedIndex<T> {
    components: HashMap<Entity, T>,
}

impl<T> DeletedIndex<T> {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
}

impl<T> DeletedIndex<T> {
    pub fn update<I, J>(&mut self, iter: I) -> HashMap<Entity, T>
    where
        I: Iterator<Item = (Entity, J)>,
        J: Hmm<Owned = T>,
    {
        let mut deleted_components = mem::replace(&mut self.components, HashMap::new());
        for (entity, q) in iter {
            let component = q.into_owned();
            deleted_components.remove(&entity);
            self.components.insert(entity, component);
        }

        deleted_components
    }
}

pub struct ChangedIndex<T> {
    components: HashMap<Entity, T>,
}

impl<T> ChangedIndex<T> {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
}

impl<T> ChangedIndex<T> {
    pub fn update<I, J>(&mut self, iter: I) -> HashMap<Entity, T>
    where
        I: Iterator<Item = (Entity, J)>,
        J: Hmm<Owned = T>,
        T: PartialEq,
    {
        let mut prev_components = mem::replace(&mut self.components, HashMap::new());
        for (entity, q) in iter {
            let component = q.into_owned();
            if let Entry::Occupied(prev_entry) = prev_components.entry(entity) {
                if prev_entry.get() == &component {
                    prev_entry.remove();
                }
            }

            self.components.insert(entity, component);
        }

        prev_components
    }
}

#[cfg(test)]
mod tests {
    use crate::system::{ChangedIndex, DeletedIndex};
    use hashbrown::HashMap;
    use hecs::World;

    macro_rules! hash_map {
        () => {
            HashMap::new()
        };

        ($($key:expr => $value:expr,)*) => {
            {
                let mut map = HashMap::new();
                $(
                    map.insert($key, $value);
                )*
                map
            }
        };

        ($key:expr => $value:expr) => {
            {
                let mut map = HashMap::new();
                map.insert($key, $value);
                map
            }
        };
    }

    #[test]
    fn test_deleted_index_tracks_entity() {
        let mut index = DeletedIndex::new();
        let mut world = World::new();

        let entity = world.spawn((1i32, "hello".to_owned()));
        assert_eq!(index.update(world.query::<(&i32, &String)>().iter()), hash_map![]);

        world.despawn(entity).unwrap();
        assert_eq!(
            index.update(world.query::<(&i32, &String)>().iter()),
            hash_map![entity => (1i32, "hello".to_owned())]
        );

        assert_eq!(index.update(world.query::<(&i32, &String)>().iter()), hash_map![]);
    }

    #[test]
    fn test_deleted_index_tracks_component() {
        let mut index = DeletedIndex::new();
        let mut world = World::new();

        let entity = world.spawn((1i32, "hello".to_owned()));
        index.update(world.query::<(&i32, &String)>().iter());

        world.remove_one::<String>(entity).unwrap();
        assert_eq!(
            index.update(world.query::<(&i32, &String)>().iter()),
            hash_map![entity => (1i32, "hello".to_owned())]
        );

        world.remove_one::<i32>(entity).unwrap();
        assert_eq!(index.update(world.query::<(&i32, &String)>().iter()), hash_map![]);

        assert_eq!(index.update(world.query::<(&i32, &String)>().iter()), hash_map![]);
    }

    #[test]
    fn test_changed_index() {
        let mut index1 = ChangedIndex::new();
        let mut index2 = ChangedIndex::new();
        let mut index3 = ChangedIndex::new();
        let mut world = World::new();

        let entity = world.spawn((1i32, "hello".to_owned()));
        assert_eq!(index1.update(world.query::<&i32>().iter()), hash_map![]);
        assert_eq!(index2.update(world.query::<(&i32, &String)>().iter()), hash_map![]);
        assert_eq!(index3.update(world.query::<&String>().iter()), hash_map![]);

        for (_, n) in world.query::<&mut i32>().iter() {
            *n += 10;
        }

        assert_eq!(index1.update(world.query::<&i32>().iter()), hash_map![entity => 1]);
        assert_eq!(
            index2.update(world.query::<(&i32, &String)>().iter()),
            hash_map![entity => (1, "hello".to_owned())]
        );
        assert_eq!(index3.update(world.query::<&String>().iter()), hash_map![]);

        assert_eq!(index1.update(world.query::<&i32>().iter()), hash_map![]);
        assert_eq!(index2.update(world.query::<(&i32, &String)>().iter()), hash_map![]);
        assert_eq!(index3.update(world.query::<&String>().iter()), hash_map![]);
    }
}
