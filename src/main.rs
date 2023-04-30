use midi2vol_mac::{midi::Connection, vol::Volume};

fn main() {
    let connection = Connection::new(0);
    let volume = Volume::new(5.0);

    for packet in connection.data.iter() {
        if packet.channel == 1 || packet.channel == 2 {
            volume.set((packet.val as f32 / 127.0 * 70.0).round() / 10.0)
        }
    }
}
