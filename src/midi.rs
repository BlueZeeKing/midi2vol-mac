use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

use coremidi::{Client, Packet, Source};

const MIDI_CHANMASK: u8 = 0x0F;

pub struct Connection {
    source_index: usize,
    thread: JoinHandle<()>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    SourceNotFound,
    ClientCannotBeCreated,
    SourceNotConnected,
    NotEnoughBytes,
    NotCCPacket,
    InputPortCannotBeCreated,
    ConnectionThreadFailure,
}

pub type PacketReceiver = Receiver<Result<CCPacket, Error>>;

impl Connection {
    pub fn new(source_index: usize) -> Result<(Self, PacketReceiver), Error> {
        let (tx, rx) = mpsc::channel::<Result<CCPacket, Error>>();

        Ok((
            Self {
                source_index,
                thread: Self::get_thread(tx, source_index),
            },
            rx,
        ))
    }

    fn get_thread(tx: Sender<Result<CCPacket, Error>>, source_index: usize) -> JoinHandle<()> {
        thread::spawn(move || {
            let client = match Client::new("Midi Vol Client") {
                Ok(client) => client,
                Err(_) => {
                    tx.send(Err(Error::ClientCannotBeCreated)).unwrap();
                    return;
                }
            };

            let tx1 = tx.clone();

            let port = match client.input_port("Midi 2 Vol Port", move |packets| {
                for packet in packets
                    .iter()
                    .filter_map(|packet| CCPacket::try_from(packet).ok())
                {
                    tx1.send(Ok(packet)).unwrap();
                }
            }) {
                Ok(port) => port,
                Err(_) => {
                    tx.send(Err(Error::InputPortCannotBeCreated)).unwrap();
                    return;
                }
            };

            match port.connect_source(match &Source::from_index(source_index) {
                Some(source) => source,
                None => {
                    tx.send(Err(Error::SourceNotFound)).unwrap();
                    return;
                }
            }) {
                Ok(_) => (),
                Err(_) => {
                    tx.send(Err(Error::SourceNotConnected)).unwrap();
                    return;
                }
            }

            thread::park();

            dbg!("stopping");
        })
    }

    fn update(&mut self) -> PacketReceiver {
        self.thread.thread().unpark();

        let (tx, rx) = mpsc::channel::<Result<CCPacket, Error>>();

        self.thread = Self::get_thread(tx, self.source_index);

        rx
    }

    pub fn set_source_index(&mut self, source_index: usize) -> PacketReceiver {
        self.source_index = source_index;
        self.update()
    }

    pub fn get_source_index(&self) -> usize {
        self.source_index
    }

    pub fn stop(&self) {
        self.thread.thread().unpark();
    }

    pub fn start(&mut self) -> PacketReceiver {
        self.update()
    }
}

#[derive(Debug, Clone)]
pub struct CCPacket {
    pub channel: u8,
    pub cc_num: u8,
    pub val: u8,
}

impl TryFrom<&[u8]> for CCPacket {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 3 {
            return Err(Error::NotEnoughBytes);
        }

        if value[0] >> 4 != 0xB {
            return Err(Error::NotCCPacket);
        }

        Ok(Self {
            channel: value[0] & MIDI_CHANMASK,
            cc_num: value[1],
            val: value[2],
        })
    }
}

impl TryFrom<&Packet> for CCPacket {
    type Error = Error;

    fn try_from(value: &Packet) -> Result<Self, Self::Error> {
        Self::try_from(value.data())
    }
}
