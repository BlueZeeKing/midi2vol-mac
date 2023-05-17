use std::{thread, time::Duration};

use coremidi::{Client, PacketBuffer};
use midi2vol_mac::{midi::Connection, vol::Volume};

#[allow(unused_variables)]
fn main() {
    thread::spawn(|| {
        let client = Client::new("example-client").unwrap();

        let source = client.virtual_source("example-source").unwrap();

        let mut val = 0;

        loop {
            source
                .received(&PacketBuffer::new(
                    0,
                    &[0xB0 | (2 & 0x07), 64 & 0x7F, val & 0x7F],
                ))
                .unwrap();

            val += 5;

            thread::sleep(Duration::from_secs(1))
        }
    });

    thread::sleep(Duration::from_secs(1));

    let volume = Volume::new(5.0, Duration::from_millis(100));

    let (connection, receiver) = Connection::new(0).unwrap();

    for packet in receiver.iter() {
        let packet = packet.unwrap();
        volume.set((packet.val as f32 / 127.0 * 70.0).round() / 10.0)
    }
}
