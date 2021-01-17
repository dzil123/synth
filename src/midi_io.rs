use std::convert::TryFrom;
use std::sync::mpsc;

use midir::{MidiInput, MidiInputConnection};
use wmidi::MidiMessage;
use wmidi::Note;

#[derive(Debug, Clone)]
pub enum SimpleMidiMessage {
    NoteOn(Note),
    NoteOff(Note),
}

pub fn open_midi_input() -> (mpsc::Receiver<SimpleMidiMessage>, MidiInputConnection<()>) {
    let input = MidiInput::new("synth").unwrap();

    let ports = input.ports();
    let (port, port_name) = ports
        .iter()
        .map(|port| (port, input.port_name(port).unwrap()))
        .inspect(|(_, name)| println!("found port {}", name))
        .collect::<Vec<_>>() // print out all ports
        .into_iter()
        .filter(|(_, name)| !name.contains("Midi Through")) // this input does nothing on my machine
        .next()
        .expect("no valid midi inputs found");
    println!("Chosen port: {}", port_name);

    let (sender, receiver) = mpsc::channel();

    let process_msg = move |midi: &[u8]| {
        let midi = MidiMessage::try_from(midi).unwrap();
        println!("midi: {:?}", midi);

        let msg = match midi {
            MidiMessage::NoteOn(_chl, note, _vel) => SimpleMidiMessage::NoteOn(note),
            MidiMessage::NoteOff(_chl, note, _vel) => SimpleMidiMessage::NoteOff(note),
            _ => return,
        };

        sender.send(msg).unwrap();
    };

    let connection = input
        .connect(
            &port,
            "synth",
            move |_time, bytes, _| process_msg(bytes),
            (),
        )
        .unwrap();

    (receiver, connection)
}
