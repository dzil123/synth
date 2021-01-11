use rodio::Source;

use crate::oscillator::Oscillator;
use crate::util::{BITRATE, BITRATE_F};

pub trait SynthTrait {
    fn _next(&mut self, osc: &Oscillator) -> f32 {
        0.0
    }

    fn next(&mut self, osc: &Oscillator) -> Option<f32> {
        Some(self._next(osc))
    }

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
        self.synth.next(&self.osc)
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
    fn _next(&mut self, osc: &Oscillator) -> f32 {
        (self)(osc)
    }
}

// why rust

// impl<T> SynthTrait for T
// where
//     T: FnMut(&Oscillator) -> Option<f32>,
// {
//     fn next(&mut self, osc: &Oscillator) -> Option<f32> {
//         (self)(osc)
//     }
// }
