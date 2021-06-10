#![allow(clippy::transmute_ptr_to_ref)]
use crate::id_map::IdMap;
use crate::widget::WidgetId;
use alloc::boxed::Box;
use core::any::TypeId;
use core::hash::Hash;
use hashbrown::HashMap;
use mopa::Any;

pub trait Property {
    type Value;
}

pub trait HasProperty<P> {}

trait PropertyStorageObject: Any {
    fn clone(&self) -> Box<dyn PropertyStorageObject>;
    fn eq(&self, other: &dyn PropertyStorageObject) -> bool;
    fn clear(&mut self);
}

mopafy!(PropertyStorageObject, core = core, alloc = alloc);

impl Clone for Box<dyn PropertyStorageObject> {
    fn clone(&self) -> Self {
        self.as_ref().clone()
    }
}

impl PartialEq for dyn PropertyStorageObject {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PropertyStorage<P>
where
    P: Property + Hash + Eq,
{
    property_map: HashMap<P, IdMap<WidgetId, P::Value>>,
}

impl<P> Default for PropertyStorage<P>
where
    P: Property + Hash + Eq + 'static,
{
    fn default() -> Self {
        Self {
            property_map: HashMap::default(),
        }
    }
}

impl<P> PropertyStorageObject for PropertyStorage<P>
where
    P: Property + Hash + Eq + Clone + 'static,
    P::Value: PartialEq + Clone,
{
    fn clone(&self) -> Box<dyn PropertyStorageObject> {
        Box::new(Clone::clone(self))
    }

    fn eq(&self, other: &dyn PropertyStorageObject) -> bool {
        other.downcast_ref().map_or(false, |other| self == other)
    }

    fn clear(&mut self) {
        for property_map in self.property_map.values_mut() {
            property_map.clear();
        }
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct PropertyMap {
    type_map: HashMap<TypeId, Box<dyn PropertyStorageObject>>,
}

impl PropertyMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        for type_map in self.type_map.values_mut() {
            type_map.clear();
        }
    }

    fn ensure_map<P>(&mut self, key: P) -> &mut IdMap<WidgetId, P::Value>
    where
        P: Property + Hash + Eq + Clone + 'static,
        P::Value: PartialEq + Clone,
    {
        let storage: &mut PropertyStorage<P> = self
            .type_map
            .entry(TypeId::of::<P>())
            .or_insert_with(|| Box::new(PropertyStorage::<P>::default()))
            .downcast_mut()
            .unwrap();

        storage.property_map.entry(key).or_default()
    }

    fn get_map<P>(&self, key: &P) -> Option<&IdMap<WidgetId, P::Value>>
    where
        P: Property + Hash + Eq + Clone + 'static,
        P::Value: PartialEq + Clone,
    {
        let storage: &PropertyStorage<P> = self.type_map.get(&TypeId::of::<P>())?.downcast_ref().unwrap();
        storage.property_map.get(key)
    }

    pub fn insert<P>(&mut self, widget_id: WidgetId, key: P, value: P::Value) -> Option<P::Value>
    where
        P: Property + Hash + Eq + Clone + 'static,
        P::Value: PartialEq + Clone,
    {
        self.ensure_map(key).insert(widget_id, value)
    }

    pub fn get<P>(&self, widget_id: WidgetId, key: &P) -> Option<&P::Value>
    where
        P: Property + Hash + Eq + Clone + 'static,
        P::Value: PartialEq + Clone,
    {
        let m = self.get_map(key)?;
        m.get(&widget_id)
    }

    pub fn iter<P>(&self, key: &P) -> Box<dyn Iterator<Item = (WidgetId, &P::Value)> + '_>
    where
        P: Property + Hash + Eq + Clone + 'static,
        P::Value: PartialEq + Clone,
    {
        if let Some(m) = self.get_map(key) {
            Box::new(m.iter().map(|(widget_id, value)| (widget_id, value)))
        } else {
            Box::new(vec![].into_iter())
        }
    }
}
