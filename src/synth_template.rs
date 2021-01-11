use std::cell::RefCell;
use std::f32::consts::{PI, TAU};
use std::panic::Location;

use rodio::Source;
use rustc_hash::FxHashMap;

use crate::util::{BITRATE, BITRATE_F};

pub trait SynthTrait {
    fn next(&mut self, osc: &Oscillator) -> f32;

    fn convert(self) -> SynthRoot<Self>
    where
        Self: Sized,
    {
        self.into()
    }
}

impl<T: SynthTrait> From<T> for SynthRoot<T> {
    fn from(v: T) -> Self {
        SynthRoot::new(v)
    }
}

pub trait SynthTraitDefault: SynthTrait + Default {
    fn create() -> SynthRoot<Self> {
        SynthRoot::default()
    }
}

impl<T: SynthTrait + Default> SynthTraitDefault for T {}

#[derive(Default, Clone)]
pub struct Oscillator {
    hashmap: RefCell<FxHashMap<Location<'static>, f32>>,
}

impl Oscillator {
    // everything here is &self even though it should be &mut self to avoid double mut borrow
    // because &mut self doesnt allow nesting like osc.get_sin(osc.get_sin(440.0))

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
}

pub struct SynthRoot<T> {
    osc: Oscillator,
    synth: T,
}

impl<T> SynthRoot<T> {
    pub fn new(synth: T) -> Self {
        Self {
            synth,
            osc: Default::default(),
        }
    }
}

impl<T: SynthTrait> Iterator for SynthRoot<T> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.synth.next(&self.osc))
    }
}

impl<T> Source for SynthRoot<T>
where
    SynthRoot<T>: Iterator<Item = f32>,
{
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        BITRATE
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl<T: Default> Default for SynthRoot<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: Clone> Clone for SynthRoot<T> {
    fn clone(&self) -> Self {
        Self {
            osc: self.osc.clone(),
            synth: self.synth.clone(),
        }
    }
}

impl<T> SynthTrait for T
where
    T: FnMut(&Oscillator) -> f32,
{
    fn next(&mut self, osc: &Oscillator) -> f32 {
        (self)(osc)
    }
}
