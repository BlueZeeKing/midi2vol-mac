use std::time::Duration;

use midi2vol_mac::{midi::Connection, vol::Volume};

#[allow(unused_variables)]
fn main() {
    let volume = Volume::new(5.0, Duration::from_millis(100));

    let data = Connection::new(0, move |packet| {
        volume.set((packet.val as f32 / 127.0 * 70.0).round() / 10.0);
        println!("{}", (packet.val as f32 / 127.0 * 70.0).round() / 10.0)
    });

    loop {}
}
