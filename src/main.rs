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
                quiet_length: 0.0,
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

#[derive(Clone)]
enum VoiceNode {
    Free(Option<usize>), // index to next free, or None if last
    Used { voice: Voice, note: Note },
}

impl VoiceNode {
    fn next(&mut self, osc: &Oscillator) -> Option<f32> {
        match self {
            Self::Used { ref mut voice, .. } => voice.next(osc),
            _ => None,
        }
    }

    fn free(&self) -> Option<Option<usize>> {
        match self {
            Self::Free(free) => Some(*free),
            _ => None,
        }
    }

    fn note(&self) -> Option<Note> {
        match self {
            Self::Used { note, .. } => Some(*note),
            _ => None,
        }
    }
}

impl Default for VoiceNode {
    fn default() -> Self {
        Self::Free(None)
    }
}

#[track_caller]
fn dbg<T: std::fmt::Debug>(v: T) {
    let loc = std::panic::Location::caller();
    println!("[{}:{}] = {:?}", loc.file(), loc.line(), v);
}

// implementation from http://gameprogrammingpatterns.com/object-pool.html#a-free-list
struct VoiceArray {
    voices: [VoiceNode; Self::SIZE],
    free: Option<usize>, // index to first free
}

impl VoiceArray {
    const SIZE: usize = 32;

    fn dbg(&self) {
        dbg((
            self.free,
            self.voices
                .iter()
                .map(|voice| voice.free())
                .collect::<Vec<_>>(),
        ));
    }

    fn next(&mut self, osc: &Oscillator) -> Option<f32> {
        let mut full_sample = 0.0;

        // this wasnt in the original implementation, but testing showed that without
        // resetting self.free to None each sample, it would create a loop in the
        // linked list, and it would itself point to or cause some node to point to a used node,
        // which should never happen
        self.free = None;

        for (i, voice) in self.voices.iter_mut().enumerate() {
            match osc.sub_osc(i, |osc| voice.next(osc)) {
                Some(sample) => full_sample += sample,
                None => {
                    *voice = VoiceNode::Free(self.free);
                    self.free = Some(i);
                }
            }
        }

        full_sample *= 0.3;
        // assert!(full_sample.abs() <= 1.0, "clipping");

        Some(full_sample)
    }

    // returns free voice and its index in the array, and marks it as used
    fn get_free(&mut self) -> (usize, &mut VoiceNode) {
        let free = self.free.unwrap(); // todo: handle None

        let voice = &mut self.voices[free];

        self.free = voice.free().unwrap();

        (free, voice)
    }
}

impl Default for VoiceArray {
    fn default() -> Self {
        let mut voices: [VoiceNode; Self::SIZE] = Default::default();

        for (i, voice) in voices.iter_mut().enumerate() {
            *voice = VoiceNode::Free(Some(i + 1));
        }

        *voices.last_mut().unwrap() = VoiceNode::Free(None);

        Self {
            voices,
            free: Some(0),
        }
    }
}

#[derive(Default)]
struct MidiSynth {
    input: MidiInput,
    voices: VoiceArray,
}

impl SynthTrait for MidiSynth {
    fn next(&mut self, osc: &Oscillator) -> Option<f32> {
        // idk whether to process all messages at once or only one per sample
        match self.input.try_recv() {
            Ok(SimpleMidiMessage::NoteOn(note)) => {
                let voice = Voice(note.to_freq_f32());
                let (i, node) = self.voices.get_free();
                *node = VoiceNode::Used { voice, note };
                osc.sub_osc(i, |osc| osc.reset());
            }
            Ok(SimpleMidiMessage::NoteOff(note)) => {
                // uhh should i use a hashmap, instead of this linear search?
                self.voices
                    .voices
                    .iter_mut()
                    .enumerate()
                    .filter(|(_, voice)| voice.note() == Some(note))
                    .for_each(|(i, _)| osc.sub_osc(i, |osc| osc.release()));
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => unreachable!(), // connection is not closed while MidiSynth is not dropped
        };

        self.voices.next(osc)
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
