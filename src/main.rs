#![feature(duration_constants)]

use rand::prelude::*;
use rodio::Source;

struct ManyChannel<T> {
    synths: Vec<T>,
    current_channel: usize,
}

impl<T: Source> ManyChannel<T>
where
    <T as Iterator>::Item: rodio::Sample,
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
    <T as Iterator>::Item: rodio::Sample,
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

#[derive(Default)]
struct Synth(f32, f32, f32);

impl Synth {
    fn new(freq: f32) -> Self {
        Synth(0.0, 0.0, freq)
    }
}

impl Iterator for Synth {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        const S: f32 = 44100.0;

        let mut x: f32 = rand::thread_rng().gen_range(-1.0..1.0);
        if self.1 > 0.5 {
            x = x.signum();
        }
        self.1 += self.0 / S;
        self.1 %= 1.0;

        self.0 += ((self.0 / 1.5) + 0.4) * (0.2 / S);

        return Some(x);
        // return Some(rand::thread_rng().gen_range(-1.0f32..1.0).signum());

        // self.0 += 880.0 / S;
        self.0 += (std::f32::consts::TAU * ((self.1.sin() * 0.5) + 1.0) * self.2) / S;
        self.1 += (std::f32::consts::TAU / 1.0) / S;
        // self.1 += (std::f32::consts::TAU / (1.0 + (self.0 % 0.5))) / S;
        // self.1 = 0.0;

        self.0 %= std::f32::consts::TAU;
        self.1 %= std::f32::consts::TAU;

        Some(self.0.sin().signum())
        // Some(0.0)
        // None
        // Some(rand::thread_rng().gen_range(-1.0..1.0))
        // Some((rand::random::<f32>() - 0.5) * 2.0)
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
        44100
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
        // Some(std::time::Duration::SECOND)
    }
}

fn play_live<T: Source + Iterator<Item = f32> + Send + 'static>(
    num_seconds: Option<u64>,
    source: T,
) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    match num_seconds {
        Some(x) => std::thread::sleep(std::time::Duration::from_secs(x)),
        None => loop {
            std::thread::sleep(std::time::Duration::MAX)
        },
    }
}

fn save_to_wav<T: Source + Iterator<Item = f32>>(filename: &str, num_seconds: f32, mut source: T) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(filename, spec).unwrap();
    for _ in 0..((44100.0 * num_seconds) as usize) {
        writer.write_sample(source.next().unwrap()).unwrap();
    }
}

fn main() {
    // let source = Synth(0.0, 1.0);
    // let source = Synth::default();
    let source = Synth::new(440.0);
    // let source = ManyChannel::new([880.0, 440.0].iter().map(|&x| Synth::new(x)).collect());
    // stream_handle.play_raw(source).unwrap();
    save_to_wav("noise.wav", 60.0, source);
}
