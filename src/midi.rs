use coremidi::{Client, InputPort, Packet, Source};

use crate::vol::Volume;

const MIDI_CHANMASK: u8 = 0x0F;

pub struct Connection {
    client: Client,
    source_index: usize,
    port: Option<InputPort>,
    volume: Volume,
}

impl Connection {
    pub fn new(source_index: usize, volume: Volume) -> Self {
        let mut new = Self {
            client: Client::new("Midi Vol Client").unwrap(),
            source_index,
            port: None,
            volume,
        };

        new.create_callback();

        new
    }

    fn create_callback(&mut self) {
        self.client = Client::new("Midi Vol Client").unwrap(); // TODO: Error handling
        let source = Source::from_index(self.source_index).unwrap();

        let volume = self.volume.clone();

        let callback =
            move |packet: CCPacket| volume.set((packet.val as f32 / 127.0 * 70.0).round() / 10.0);

        self.port = Some(
            self.client
                .input_port("Midi Vol Port", move |packets| {
                    for packet in packets
                        .iter()
                        .filter_map(|packet| CCPacket::try_from(packet).ok())
                    {
                        callback(packet)
                    }
                })
                .unwrap(),
        );

        self.port.as_ref().unwrap().connect_source(&source).unwrap();
    }

    pub fn set_source_index(&mut self, source_index: usize) {
        self.source_index = source_index;

        self.create_callback();
    }
}

#[derive(Debug)]
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

pub enum Error {
    NotEnoughBytes,
    NotCCPacket,
}
