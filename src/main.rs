#![allow(unused_imports, dead_code)]

mod adsr;
mod audio_util;
mod manychannel;
mod oscillator;
mod synth_template;
mod util;

use crate::adsr::{ADSRParams, ADSR};
use crate::audio_util::{play_live, save_to_wav};
use crate::oscillator::Oscillator;
use crate::synth_template::{SynthTrait, SynthTraitDefault};
use crate::util::{distort, lerp, scale};

#[derive(Clone)]
struct Synth {
    adsr: ADSR,
}

impl SynthTrait for Synth {
    fn _next(&mut self, osc: &Oscillator) -> f32 {
        let amplitude = self.adsr.next().unwrap_or_else(|| {
            self.adsr.reset();
            0.0
        });

        let ratio = 18.0;
        let freq = 440.0;
        // let freq = scale(
        //     if amplitude == 0.0 {
        //         0.0
        //     } else {
        //         osc.get_tri(0.071384612348723)
        //     },
        //     freq / 1.1,
        //     freq * 1.1,
        // );

        // let amnt = 4.0;
        // osc.get_sin(scale(osc.get_sin(freq * ratio), freq / amnt, freq * amnt)) * amplitude

        if osc.rising_edge(amplitude - 0.001) {
            // println!("{}", osc.get_saw(0.1));
            println!("{}", osc.incrementing());
        }

        let amnt = 8.0; // number of halfsteps
        let amnt = scale(osc.get_sin(0.1), 1.0, 100.0);
        const A: f32 = 1.059463094359;
        osc.get_sin(scale(
            osc.get_sin(freq * ratio),
            freq * A.powf(-amnt),
            freq * A.powf(amnt),
        )) * amplitude
    }
}

impl Default for Synth {
    fn default() -> Self {
        Self {
            adsr: ADSRParams::flat2(0.2, 0.1).build()
            // adsr: ADSRParams {
            //     attack_length: 0.003,
            //     decay_length: 0.008,
            //     sustain_percent: 0.2,
            //     sustain_length: 0.0,
            //     release_length: 0.3,
            //     quiet_length: 0.0,
            // }
            // .build(),
            // adsr: ADSR::default(),
        }
    }
}

fn main() {
    let new_synth = Synth::create;

    // save_to_wav(new_synth(), "output.wav", 2.0);
    play_live(new_synth(), None);
}
