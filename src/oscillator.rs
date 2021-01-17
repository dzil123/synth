use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::f32::consts::{PI, TAU};
use std::ops::DerefMut;
use std::panic::Location;

use rustc_hash::FxHashMap;

use crate::util::{BITRATE, BITRATE_F};
use crate::{ADSRParams, ADSR};

type HashMap<T> = FxHashMap<Location<'static>, T>;

struct AnyHashMap {
    inner: Box<dyn Any + Send>,
    clone_func: Box<dyn Fn(&Self) -> Self + Send>,
}

impl AnyHashMap {
    fn default<T: Any + Default + Clone + Send>() -> Self {
        Self::new::<T>(HashMap::<T>::default())
    }

    fn new<T: Any + Default + Clone + Send>(val: HashMap<T>) -> Self {
        let clone_func = |v: &Self| Self::new(v.downcast_ref::<T>().clone());

        Self {
            inner: Box::new(val),
            clone_func: Box::new(clone_func),
        }
    }

    fn downcast_ref<T: Any + Default + Clone + Send>(&self) -> &HashMap<T> {
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

    // returns &mut HashMap<T> borrowed over the entirety of self.hashmap_meta
    // any attempt to borrow self.hashmap_meta before this is dropped will panic
    fn hashmap_mut<T>(&self) -> RefMut<HashMap<T>>
    where
        T: Any + Default + Clone + Send,
    {
        let hashmap_meta = self.hashmap_meta.borrow_mut();

        let hashmap = RefMut::map(hashmap_meta, |hashmap_meta| {
            hashmap_meta
                .entry(TypeId::of::<T>())
                .or_insert_with(|| AnyHashMap::default::<T>())
        });

        let hashmap = RefMut::map(hashmap, |hashmap| {
            hashmap.inner.downcast_mut::<HashMap<T>>().unwrap()
        });

        hashmap
    }

    // returns &HashMap<T>, borrowed only immutably
    // panics if hashmap_meta does not already include an entry for T
    // ensure it does beforehand with hashmap_mut()
    fn hashmap_ref<T>(&self) -> Ref<HashMap<T>>
    where
        T: Any + Default + Clone + Send,
    {
        let hashmap_meta = self.hashmap_meta.borrow();

        let hashmap = Ref::map(hashmap_meta, |hashmap_meta| {
            hashmap_meta.get(&TypeId::of::<T>()).unwrap()
        });

        let hashmap = Ref::map(hashmap, |hashmap| hashmap.downcast_ref::<T>());

        hashmap
    }

    #[track_caller]
    fn unique_caller<T, U>(&self, default: U, modify: T) -> U
    where
        T: FnOnce(&mut U),
        U: Copy + Default + Send + 'static,
    {
        let loc = Location::caller();

        *self
            .hashmap_mut()
            .entry(*loc)
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
    pub fn incrementing(&self) -> u32 {
        self.unique_caller(0, |v| *v += 1)
    }

    #[track_caller]
    pub fn adsr(&self, adsr_params: ADSRParams) -> ADSRImposter {
        let loc = Location::caller();
        let mut hashmap: RefMut<HashMap<RefCell<ADSR>>> = self.hashmap_mut();

        if hashmap.get(loc).is_none() {
            hashmap.insert(*loc, RefCell::new(adsr_params.build()));
        }

        ADSRImposter(self, loc)
    }

    fn adsr_impl<T, U>(&self, loc: &Location<'static>, func: T) -> U
    where
        T: FnOnce(&mut ADSR) -> U,
    {
        let hashmap: Ref<HashMap<RefCell<ADSR>>> = self.hashmap_ref();
        let adsr = Ref::map(hashmap, |hashmap| hashmap.get(loc).unwrap());
        let mut adsr = adsr.borrow_mut();

        func(adsr.deref_mut())
    }

    pub fn release(&self) {
        // todo: mark certain adsrs / oscillators as per-note, as opposed to lfo
        for (_, adsr) in self.hashmap_mut::<RefCell<ADSR>>().iter_mut() {
            adsr.borrow_mut().release();
        }
    }

    pub fn reset(&self) {
        self.hashmap_meta.borrow_mut().clear();
    }
}

pub struct ADSRImposter<'a>(&'a Oscillator, &'static Location<'static>);

impl<'a> ADSRImposter<'a> {
    fn inner<T: FnOnce(&mut ADSR) -> U, U>(&self, func: T) -> U {
        self.0.adsr_impl(self.1, func)
    }

    pub fn reset(&mut self) {
        self.inner(|adsr| adsr.reset())
    }

    pub fn is_end(&self) -> bool {
        self.inner(|adsr| adsr.is_end())
    }

    // if true, the ending of this envelope can be cut short (interrupted)
    pub fn is_done(&self) -> bool {
        self.inner(|adsr| adsr.is_done())
    }

    pub fn release(&mut self) {
        self.inner(|adsr| adsr.release())
    }

    pub fn next(&mut self) -> Option<f32> {
        self.inner(|adsr| adsr.next())
    }
}
