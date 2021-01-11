use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::f32::consts::{PI, TAU};
use std::panic::Location;

use rustc_hash::FxHashMap;

use crate::util::{BITRATE, BITRATE_F};

#[derive(Default, Clone)]
pub struct Oscillator {
    hashmap: RefCell<FxHashMap<Location<'static>, f32>>,
}

impl Oscillator {
    // everything here is &self even though it should be &mut self to avoid double mut borrow
    // because &mut self doesnt allow nesting like osc.get_sin(osc.get_sin(440.0))

    #[track_caller]
    fn unique_caller<T: FnOnce(&mut f32)>(&self, default: f32, modify: T) -> f32 {
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
        *self
            .hashmap
            .borrow_mut()
            .entry(*Location::caller())
            .and_modify(|v| {
                *v += freq * len / BITRATE_F;
                *v %= len
            })
            .or_insert(start - low)
            + low
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

    // #[track_caller]
}
