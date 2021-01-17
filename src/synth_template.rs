use std::cell::RefCell;
use std::time::Instant;

use rodio::Source;

use crate::oscillator::Oscillator;
use crate::util::{BITRATE, BITRATE_F};

pub trait SynthTrait {
    fn _next(&mut self, _osc: &Oscillator) -> f32 {
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

#[derive(Clone)]
struct InnerMut {
    last_called: Instant,
    count: u32,
}

impl Default for InnerMut {
    fn default() -> Self {
        Self {
            last_called: Instant::now(),
            count: 0,
        }
    }
}

impl InnerMut {
    fn update_time(&mut self) {
        // let now = Instant::now();
        // let duration = now.duration_since(self.last_called);
        // self.last_called = now;
        // println!("update {:?}", duration);

        const TIME: std::time::Duration = std::time::Duration::from_millis(10);

        self.count += 1;
        let now = Instant::now();
        if now.duration_since(self.last_called) >= TIME {
            self.last_called = now;
            println!("update {:?} {}", TIME, self.count);

            self.count = 0;
        }
    }
}

pub struct SynthRoot<T> {
    osc: Oscillator,
    synth: T,
    inner_mut: RefCell<InnerMut>,
}

impl<T> SynthRoot<T> {
    pub fn new(synth: T) -> Self {
        Self {
            synth,
            osc: Default::default(),
            inner_mut: RefCell::new(Default::default()),
        }
    }
}

impl<T: SynthTrait> Iterator for SynthRoot<T> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // self.inner_mut.borrow_mut().update_time();
        // None
        self.synth.next(&self.osc)
    }
}

impl<T> Source for SynthRoot<T>
where
    SynthRoot<T>: Iterator<Item = f32>,
{
    fn current_frame_len(&self) -> Option<usize> {
        // Some(BITRATE as usize * 10)
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
        // Some(std::time::Duration::new(1, 0))
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
            inner_mut: self.inner_mut.clone(),
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
