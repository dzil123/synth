#![feature(duration_constants)]

fn main() {
    dbg!(cpal::available_hosts());
    let x = cpal::default_host()
        // .unwrap()
        .devices()
        .unwrap()
        .map(|x| x.name())
        .collect::<Vec<_>>();
    dbg!(x);
    dbg!(cpal::default_host().default_output_device().unwrap().name());
    let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    // Load a sound from a file, using a path relative to Cargo.toml
    // let file = File::open("sound.ogg").unwrap();
    // let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    // stream_handle.play_raw(source.convert_samples());

    loop {
        std::thread::sleep(std::time::Duration::MAX);
    }
}
