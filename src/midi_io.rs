use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc;

use midir::MidiInputConnection;
use wmidi::{MidiMessage, Note};

#[derive(Debug, Clone)]
pub enum SimpleMidiMessage {
    NoteOn(Note),
    NoteOff(Note),
}

type Receiver = mpsc::Receiver<SimpleMidiMessage>;

pub struct MidiInput {
    pub receiver: Receiver,
    pub connection: MidiInputConnection<()>,
}

impl MidiInput {
    pub fn new(name_filter: Option<&str>) -> Self {
        let input = midir::MidiInput::new("synth").unwrap();

        let ports = input.ports();
        let (port, port_name) = ports
            .iter()
            .map(|port| (port, input.port_name(port).unwrap()))
            .inspect(|(_, name)| println!("found port {}", name))
            .collect::<Vec<_>>() // print out all ports
            .into_iter()
            .filter(|(_, name)| !name.contains("Midi Through")) // this input does nothing on my machine
            .filter(|(_, name)| name_filter.map(|f| name.contains(f)).unwrap_or(true))
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

        Self {
            receiver,
            connection,
        }
    }
}

impl Default for MidiInput {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Deref for MidiInput {
    type Target = Receiver;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}

impl DerefMut for MidiInput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.receiver
    }
}
