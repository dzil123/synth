use rand::Rng;
use rodio::Source;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::f32::consts::{PI, TAU};
use std::panic::Location;

mod adsr;
mod manychannel;
mod util;

use util::*;

#[derive(Default)]
struct Synth {
    hashmap: RefCell<FxHashMap<Location<'static>, f32>>,
}

impl Synth {
    fn new() -> Self {
        Synth {
            ..Default::default()
        }
    }

    #[track_caller]
    fn get(&self, freq: f32, start: f32, low: f32, high: f32) -> f32 {
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
    fn get_sin(&self, freq: f32) -> f32 {
        self.get(freq, 0.0, 0.0, TAU).sin()
    }

    #[track_caller]
    fn get_tri(&self, freq: f32) -> f32 {
        let x = self.get(freq, 0.0, -1.0, 3.0);

        if x > 1.0 {
            2.0 - x
        } else {
            x
        }
    }

    #[track_caller]
    fn get_saw(&self, freq: f32) -> f32 {
        self.get(freq, 0.0, -1.0, 1.0)
    }

    fn _next(&mut self) -> f32 {
        // self.get_sin(scale(self.get_sin(9.0 / 7.0), 440.0, 660.0))
        //     * self.get_sin(scale(self.get_sin(11.0 / 7.0), 350.0, 243.123))
        distort(self.get_sin(440.0), scale(self.get_sin(0.5), 0.0, 1.0))
    }
}

impl Iterator for Synth {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self._next())
    }
}

impl Source for Synth {
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

fn play_live<T: Source + Iterator<Item = f32> + Send + 'static>(
    source: T,
    num_seconds: Option<u64>,
) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    match num_seconds {
        Some(x) => std::thread::sleep(std::time::Duration::from_secs(x)),
        None => loop {
            std::thread::sleep(std::time::Duration::new(u64::MAX, 0))
        },
    }
}

fn save_to_wav<T: Source + Iterator<Item = f32>>(mut source: T, filename: &str, num_seconds: f32) {
    let spec = hound::WavSpec {
        channels: source.channels(),
        sample_rate: BITRATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(filename, spec).unwrap();
    for x in 0..((BITRATE_F * num_seconds) as _) {
        writer
            .write_sample(match source.next() {
                Some(x) => x,
                None => {
                    println!("source ended early at {} sec", (x as f32) / BITRATE_F);
                    break;
                }
            })
            .unwrap();

        if (x % (BITRATE * 5)) == 0 {
            println!("{}...", x / BITRATE);
        }
    }

    writer.finalize().unwrap();
}

fn main() {
    let new_source = || Synth::new();

    // save_to_wav(new_source(), "saw440.wav", 10.0);
    play_live(new_source(), None);
}
