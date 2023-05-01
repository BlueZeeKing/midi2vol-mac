use coremidi::{Client, InputPort, Packet, Source};

const MIDI_CHANMASK: u8 = 0x0F;

pub struct Connection<F: FnMut(CCPacket) + Send + 'static + Clone> {
    client: Client,
    callback: F,
    source_index: usize,
    port: Option<InputPort>,
}

impl<F: FnMut(CCPacket) + Send + 'static + Clone> Connection<F> {
    pub fn new(source_index: usize, callback: F) -> Self {
        let mut new = Self {
            client: Client::new("Midi Vol Client").unwrap(),
            callback,
            source_index,
            port: None,
        };

        new.create_callback();

        new
    }

    fn create_callback(&mut self) {
        self.client = Client::new("Midi Vol Client").unwrap(); // TODO: Error handling
        let source = Source::from_index(self.source_index).unwrap();

        let mut callback = self.callback.clone();

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
