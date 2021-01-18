#![allow(unused_imports, dead_code)]

use std::sync::mpsc;

use wmidi::Note;

mod adsr;
mod audio_util;
mod manychannel;
mod midi_io;
mod oscillator;
mod synth_template;
mod util;

use crate::adsr::{ADSRParams, ADSR};
use crate::audio_util::{play_live, save_to_wav};
use crate::midi_io::{MidiInput, SimpleMidiMessage};
use crate::oscillator::Oscillator;
use crate::synth_template::{SynthTrait, SynthTraitDefault};
use crate::util::{distort, lerp, scale};

#[derive(Default, Clone)]
struct Voice(f32);

impl SynthTrait for Voice {
    fn next(&mut self, osc: &Oscillator) -> Option<f32> {
        let vol = osc
            .adsr(ADSRParams {
                ..Default::default()
            })
            .next()?;

        let out = osc.get_sin(self.0).signum() * vol;

        Some(out)
    }
}

struct Synth<T: Iterator<Item = Note>> {
    voice: Option<Voice>,
    notes: T,
}

impl<T: Iterator<Item = Note>> Synth<T> {
    fn new(notes: T) -> Self {
        Self { voice: None, notes }
    }
}

impl<T: Iterator<Item = Note>> SynthTrait for Synth<T> {
    fn next(&mut self, osc: &Oscillator) -> Option<f32> {
        loop {
            if let Some(voice) = self.voice.as_mut() {
                let out = voice.next(osc);
                if out.is_some() {
                    break out;
                }
                self.voice.take();
                osc.reset();
            }

            if let Some(note) = self.notes.next() {
                self.voice.replace(Voice(note.to_freq_f32()));
                continue;
            }

            break None;
        }
    }
}

#[derive(Default)]
struct MidiSynth {
    input: MidiInput,
    voice: Option<Voice>,
    current_note: Option<Note>,
}

impl SynthTrait for MidiSynth {
    fn next(&mut self, osc: &Oscillator) -> Option<f32> {
        // idk whether to process all messages at once or only one per sample
        match self.input.try_recv() {
            Ok(SimpleMidiMessage::NoteOn(note)) => {
                dbg!();
                osc.reset();

                self.voice.replace(Voice(note.to_freq_f32()));
                self.current_note.replace(note);
            }
            Ok(SimpleMidiMessage::NoteOff(note)) if self.current_note == Some(note) => {
                dbg!();
                // self.voice.take();
                osc.release();
                // self.current_note.take();
            }
            Ok(SimpleMidiMessage::NoteOff(_)) => dbg!(),
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => unreachable!(), // connection is not closed while MidiSynth is not dropped
        };

        if let Some(voice) = self.voice.as_mut() {
            let out = voice.next(osc);
            if out.is_some() {
                return out;
            }
            self.voice.take();
        }

        Some(0.0)
    }
}

fn notes() -> impl Iterator<Item = Note> {
    use Note::*;

    vec![C5, D5, E5, F5, G5, A5, B5, C6].into_iter().cycle()
}

fn main() {
    // let new_synth = || Synth::new(notes()).convert();
    let new_synth = MidiSynth::create;

    // save_to_wav(new_synth(), "output.wav", 2.0);
    play_live(new_synth(), None);
}
