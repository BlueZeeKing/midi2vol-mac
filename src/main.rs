use std::{thread, time::Duration};

use coremidi::{Client, PacketBuffer};
use midi2vol_mac::{midi::Connection, vol::Volume, MIDI2Vol};

#[allow(unused_variables)]
fn main() {
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

        if val > 127 {
            val = 0
        }

        thread::sleep(Duration::from_secs(1))
    }
}
