use alloc::vec::Vec;
use bit_vec::BitVec;
use core::iter::FromIterator;
use core::marker::PhantomData;
use core::mem;
use core::mem::MaybeUninit;

#[derive(Debug)]
pub struct IdMap<K, V> {
    valid: BitVec,
    keys: PhantomData<K>,
    values: Vec<MaybeUninit<V>>,
}

impl<K, V> Default for IdMap<K, V> {
    fn default() -> Self {
        Self {
            valid: BitVec::new(),
            keys: PhantomData,
            values: Vec::new(),
        }
    }
}

impl<K, V> Clone for IdMap<K, V>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        let valid = self.valid.clone();

        let values = self
            .opt_values()
            .map(|value| value.cloned().map_or_else(MaybeUninit::uninit, MaybeUninit::new))
            .collect();

        Self {
            valid,
            keys: PhantomData,
            values,
        }
    }
}

impl<K, V> PartialEq for IdMap<K, V>
where
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.opt_values().eq(other.opt_values())
    }
}

impl<K, V> IdMap<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.valid.reserve(additional);
        self.values.reserve(additional);
    }

    pub fn clear(&mut self) {
        for (value, valid) in self.values.drain(..).zip(self.valid.iter()) {
            if valid {
                let value = unsafe { value.assume_init() };
                mem::drop(value);
            }
        }

        self.valid.clear();
    }

    fn opt_values(&self) -> impl Iterator<Item = Option<&V>> {
        self.values
            .iter()
            .zip(self.valid.iter())
            .map(|(value, valid)| valid.then(|| unsafe { &*value.as_ptr() }))
    }
}

impl<K, V> IdMap<K, V>
where
    K: salsa::InternKey,
{
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let key = key.as_intern_id().as_usize();
        let value = MaybeUninit::new(value);
        let len = key + 1;
        if let Some(n) = len.checked_sub(self.valid.len()) {
            self.valid.grow(n, false);
        }

        if len > self.values.len() {
            self.values.resize_with(len, MaybeUninit::uninit);
        }

        if self.valid[key] {
            let prev_value = mem::replace(&mut self.values[key], value);
            Some(unsafe { prev_value.assume_init() })
        } else {
            self.valid.set(key, true);
            self.values[key] = value;
            None
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let key = key.as_intern_id().as_usize();
        let valid = self.valid.get(key)?;
        valid.then(|| {
            let value = &self.values[key];
            unsafe { &*value.as_ptr() }
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.opt_values()
            .enumerate()
            .filter_map(|(key, opt_value)| Some((K::from_intern_id(salsa::InternId::from(key)), opt_value?)))
    }
}

impl<K, V> FromIterator<(K, V)> for IdMap<K, V>
where
    K: salsa::InternKey,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        let mut this = Self::new();
        this.extend(iter);
        this
    }
}

impl<K, V> Extend<(K, V)> for IdMap<K, V>
where
    K: salsa::InternKey,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (K, V)>,
    {
        let iter = iter.into_iter();
        let len = iter.size_hint().1.unwrap_or_default();
        self.reserve(len);
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}
