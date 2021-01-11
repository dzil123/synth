use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::f32::consts::{PI, TAU};
use std::panic::Location;

use rustc_hash::FxHashMap;

use crate::util::{AnyClone, BITRATE, BITRATE_F};

type HashMap<T> = FxHashMap<Location<'static>, T>;

struct AnyHashMap {
    inner: Box<dyn Any + Send>,
    clone_func: Box<dyn Fn(&Self) -> Self + Send + Sync>,
}

impl AnyHashMap {
    fn default<T: Any + Default + Clone + Send + Sync>() -> Self {
        Self::new::<T>(HashMap::<T>::default())
    }

    fn new<T: Any + Default + Clone + Send + Sync>(val: HashMap<T>) -> Self {
        let clone_func = |v: &Self| Self::new(v.downcast_ref::<T>().clone());

        Self {
            inner: Box::new(val),
            clone_func: Box::new(clone_func),
        }
    }

    fn downcast_ref<T: Any + Default + Clone + Send + Sync>(&self) -> &HashMap<T> {
        self.inner.downcast_ref::<HashMap<T>>().unwrap()
    }
}

impl Clone for AnyHashMap {
    fn clone(&self) -> Self {
        (self.clone_func)(self)
    }
}

#[derive(Default, Clone)]
pub struct Oscillator {
    hashmap_meta: RefCell<FxHashMap<TypeId, AnyHashMap>>, // effective signature: HashMap<T::TypeId, HashMap<T>>
}

impl Oscillator {
    // everything here is &self even though it should be &mut self to avoid double mut borrow
    // because &mut self doesnt allow nesting like osc.get_sin(osc.get_sin(440.0))

    fn hashmap<T, U, V>(&self, func: U) -> V
    where
        T: Any + Default + Clone + Send + Sync,
        U: FnOnce(&mut HashMap<T>) -> V,
    {
        let mut hashmap_meta = self.hashmap_meta.borrow_mut();
        let hashmap = hashmap_meta
            .entry(TypeId::of::<T>())
            .or_insert_with(|| AnyHashMap::default::<T>());

        let hashmap = hashmap.inner.downcast_mut::<HashMap<T>>().unwrap();

        func(hashmap)
    }

    #[track_caller]
    fn unique_caller<T, U>(&self, default: U, modify: T) -> U
    where
        T: FnOnce(&mut U),
        U: Copy + Default + Send + Sync + 'static,
    {
        let loc = Location::caller();

        self.hashmap(|hashmap| *hashmap.entry(*loc).and_modify(modify).or_insert(default))
    }

    #[track_caller]
    pub fn get(&self, freq: f32, start: f32, low: f32, high: f32) -> f32 {
        // value is stored between 0 and len
        let len = high - low;
        self.unique_caller(start - low, |v| {
            *v += freq * len / BITRATE_F;
            *v %= len
        }) + low
    }

    #[track_caller]
    pub fn get_sin(&self, freq: f32) -> f32 {
        self.get(freq, 0.0, 0.0, TAU).sin()
    }

    #[track_caller]
    pub fn get_tri(&self, freq: f32) -> f32 {
        let x = self.get(freq, 0.0, -1.0, 3.0);

        if x > 1.0 {
            2.0 - x
        } else {
            x
        }
    }

    #[track_caller]
    pub fn get_saw(&self, freq: f32) -> f32 {
        self.get(freq, 0.0, -1.0, 1.0)
    }

    #[track_caller]
    pub fn rising_edge(&self, val: f32) -> bool {
        let mut last_signum = 0.0;
        self.unique_caller(-1.0, |v| {
            last_signum = *v;
            *v = val.signum();
        });

        last_signum < 0.0 && val.signum() >= 0.0
    }

    #[track_caller]
    pub fn incrementing(&self) -> u32 {
        self.unique_caller(0, |v| *v += 1)
    }
}
