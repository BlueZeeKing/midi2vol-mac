use coremidi::{Client, InputPort, Packet, Source};

const MIDI_CHANMASK: u8 = 0x0F;

pub fn new<F: FnMut(CCPacket) + Send + 'static>(
    source_index: usize,
    mut callback: F,
) -> (Client, InputPort) {
    let client = Client::new("Midi Vol Client").unwrap(); // TODO: Error handling
    let source = Source::from_index(source_index).unwrap();

    let port = client
        .input_port("Midi Vol Port", move |packets| {
            for packet in packets
                .iter()
                .filter_map(|packet| CCPacket::try_from(packet).ok())
            {
                callback(packet)
            }
        })
        .unwrap();

    port.connect_source(&source).unwrap();

    (client, port)
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
