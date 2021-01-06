use rand::Rng;
use rodio::Source;
use rustc_hash::FxHashMap;
use std::f32::consts::{PI, TAU};
use std::panic::Location;

const BITRATE: u32 = 44100;
const BITRATE_F: f32 = BITRATE as _;

// assuming each synth is only 1 channel
struct ManyChannel<T> {
    synths: Vec<T>,
    current_channel: usize,
}

impl<T: Source> ManyChannel<T>
where
    T::Item: rodio::Sample,
{
    fn new(synths: Vec<T>) -> Self {
        Self {
            synths,
            current_channel: 0,
        }
    }
}

impl<T: Source> Iterator for ManyChannel<T>
where
    T::Item: rodio::Sample,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.synths[self.current_channel].next();
        self.current_channel = (self.current_channel + 1) % self.synths.len();
        result
    }
}

impl<T: Source> Source for ManyChannel<T>
where
    <T as Iterator>::Item: rodio::Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.synths[self.current_channel].current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.synths.len() as _
    }

    fn sample_rate(&self) -> u32 {
        self.synths[self.current_channel].sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.synths[self.current_channel].total_duration()
    }
}

// scale (-1 < x < 1) to (a < ans < b)
fn scale(x: f32, a: f32, b: f32) -> f32 {
    (b + a + x * (b - a)) / 2.0
}

#[derive(Default)]
struct Synth {
    hashmap: FxHashMap<Location<'static>, f32>,
}

impl Synth {
    fn new() -> Self {
        Synth {
            ..Default::default()
        }
    }

    #[track_caller]
    fn get(&mut self, period: f32, start: f32, low: f32, high: f32) -> f32 {
        // value is stored between 0 and len
        let len = high - low;
        *self
            .hashmap
            .entry(*Location::caller())
            .and_modify(|v| {
                *v += period * len / BITRATE_F;
                *v %= len
            })
            .or_insert(start - low)
            + low
    }

    #[track_caller]
    fn get_sin(&mut self, period: f32) -> f32 {
        self.get(period, 0.0, 0.0, TAU).sin()
    }

    fn _next(&mut self) -> f32 {
        let lfo = self.get_sin(10.0);
        self.get_sin(scale(lfo, 440.0, 660.0))
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

    // save_to_wav(new_source(), "sin440.wav", 1.0);
    play_live(new_source(), None);
}
