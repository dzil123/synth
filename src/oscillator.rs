use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::f32::consts::{PI, TAU};
use std::panic::Location;

use rustc_hash::FxHashMap;

use crate::util::{AnyClone, BITRATE, BITRATE_F};

// type HashMap<T> = RefCell<FxHashMap<Location<'static>, T>>;
type HashMap<T> = FxHashMap<Location<'static>, T>;

#[derive(Default)]
pub struct Oscillator {
    // hashmap: RefCell<FxHashMap<Location<'static>, f32>>,
    hashmap: RefCell<HashMap<f32>>,
    hashmap_meta: RefCell<FxHashMap<TypeId, Box<dyn Any + Send>>>,
}

impl Oscillator {
    // everything here is &self even though it should be &mut self to avoid double mut borrow
    // because &mut self doesnt allow nesting like osc.get_sin(osc.get_sin(440.0))

    // fn hashmap<T, U, V, X>(&self, func: U, default: X) -> V
    // where
    //     T: Any + Default + Send + 'static,
    //     // Box<(dyn Any + Send + 'static)>: Default, // this requires #![feature(trivial_bounds)], to allow entry.or_default()
    //     U: FnOnce(&mut HashMap<T>) -> V,
    //     X: FnOnce() -> HashMap<T>,
    // {
    fn hashmap<T, U, V>(&self, func: U) -> V
    where
        T: Any + Default + Send + 'static,
        // Box<(dyn Any + Send + 'static)>: Default, // this requires #![feature(trivial_bounds)], to allow entry.or_default()
        U: FnOnce(&mut HashMap<T>) -> V,
    {
        let mut hashmap_meta = self.hashmap_meta.borrow_mut();
        // let hashmap = hashmap_meta.entry(TypeId::of::<T>()).or_default();
        // let hashmap = hashmap_meta
        //     .entry(TypeId::of::<T>())
        //     .or_insert_with(|| Default::default());
        let hashmap = hashmap_meta
            .entry(TypeId::of::<T>())
            // .or_insert_with(|| Box::new(default()));
            .or_insert_with(|| Box::new(HashMap::<T>::default()));

        // println!("{}", std::any::type_name_of_val(&hashmap));

        let hashmap: Option<&mut HashMap<T>> = hashmap.downcast_mut();
        let hashmap = hashmap.unwrap();

        func(hashmap)
    }

    #[track_caller]
    fn unique_caller<T: FnOnce(&mut f32)>(&self, default: f32, modify: T) -> f32 {
        // self.hashmap(|_: &mut HashMap<f32>| 0.0, || Default::default());
        self.hashmap(|_: &mut HashMap<f32>| 0.0);

        *self
            .hashmap
            .borrow_mut()
            .entry(*Location::caller())
            .and_modify(modify)
            .or_insert(default)
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
    pub fn incrementing(&self) -> f32 {
        self.unique_caller(123456789.1, |v| *v += 1.0)
    }
}
