#![allow(unused_imports, dead_code)]

mod adsr;
mod audio_util;
mod manychannel;
mod synth_template;
mod util;

use crate::audio_util::{play_live, save_to_wav};
use crate::synth_template::{Oscillator, SynthTrait, SynthTraitDefault};
use crate::util::{distort, scale};

#[derive(Default)]
struct Synth;

impl SynthTrait for Synth {
    fn next(&mut self, osc: &Oscillator) -> f32 {
        distort(osc.get_sin(440.0), scale(osc.get_sin(0.5), 0.0, 1.0))
    }
}

fn main() {
    let new_synth = Synth::create;

    // save_to_wav(new_synth(), "output.wav", 10.0);
    play_live(new_synth(), None);
}
