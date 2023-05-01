use coremidi::{Client, InputPort, Packet, Source};

use crate::vol::Volume;

const MIDI_CHANMASK: u8 = 0x0F;

pub struct Connection {
    client: Client,
    port: Result<InputPort, Error>,

    source_index: usize,
    pub volume: Volume,
}

#[derive(Debug)]
pub enum Error {
    NotCreatedYet,
    SourceNotFound,
    ClientCannotBeCreated,
    SourceNotConnected,
    NotEnoughBytes,
    NotCCPacket,
    InputPortCannotBeCreated,
}

impl Connection {
    pub fn new(source_index: usize, volume: Volume) -> Result<Self, Error> {
        let mut new = Self {
            client: match Client::new("Midi Vol Client") {
                Ok(client) => client,
                Err(_) => return Err(Error::ClientCannotBeCreated),
            },
            source_index,
            port: Err(Error::NotCreatedYet),
            volume,
        };

        new.port = new.create_callback();

        Ok(new)
    }

    pub fn set_port(&mut self, port: Result<InputPort, Error>) {
        self.port = port;
    }

    pub fn create_callback(&mut self) -> Result<InputPort, Error> {
        let source = match Source::from_index(self.source_index) {
            Some(source) => source,
            None => return Err(Error::SourceNotFound),
        };

        let volume = self.volume.clone();

        let callback =
            move |packet: CCPacket| volume.set((packet.val as f32 / 127.0 * 70.0).round() / 10.0);

        let port = match self.client.input_port("Midi Vol Port", move |packets| {
            for packet in packets
                .iter()
                .filter_map(|packet| CCPacket::try_from(packet).ok())
            {
                callback(packet)
            }
        }) {
            Ok(port) => port,
            Err(_) => return Err(Error::InputPortCannotBeCreated),
        };

        match port.connect_source(&source) {
            Err(_) => return Err(Error::SourceNotConnected),
            _ => (),
        };

        Ok(port)
    }

    pub fn set_source_index(&mut self, source_index: usize) {
        self.source_index = source_index;

        self.port = self.create_callback();
    }

    pub fn get_error(&self) -> Option<&Error> {
        match &self.port {
            Ok(_) => None,
            Err(err) => Some(err),
        }
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
