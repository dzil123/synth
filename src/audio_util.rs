use std::thread::sleep;
use std::time::Duration;

use hound::Result as HoundResult;
use rodio::Source;

use crate::util::{BITRATE, BITRATE_F};

pub fn play_live<T>(source: T, num_seconds: Option<u64>)
where
    T: Source + Iterator<Item = f32> + Send + 'static,
{
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    match num_seconds {
        Some(x) => sleep(Duration::from_secs(x)),
        None => loop {
            sleep(Duration::new(u64::MAX, 0))
        },
    }
}

pub fn save_to_wav<T>(mut source: T, filename: &str, num_seconds: f32) -> HoundResult<()>
where
    T: Source + Iterator<Item = f32>,
{
    let spec = hound::WavSpec {
        channels: source.channels(),
        sample_rate: BITRATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(filename, spec)?;
    for x in 0..((BITRATE_F * num_seconds) as _) {
        writer.write_sample(match source.next() {
            Some(x) => x,
            None => {
                println!("source ended early at {} sec", (x as f32) / BITRATE_F);
                break;
            }
        })?;

        if (x % (BITRATE * 5)) == 0 {
            println!("{}...", x / BITRATE);
        }
    }

    writer.finalize()?;
    Ok(())
}
