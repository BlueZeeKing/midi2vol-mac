use std::{
    process::Command,
    sync::{atomic::Ordering, Arc},
    thread,
    time::Duration,
};

use atomic_float::AtomicF32;

pub struct Volume {
    volume: Arc<AtomicF32>,
}

impl Volume {
    pub fn new(volume: f32) -> Self {
        let volume = Arc::new(AtomicF32::new(volume));

        let volume2 = volume.clone();

        thread::spawn(move || {
            let mut old = 11f32;
            loop {
                let volume = volume2.load(Ordering::Relaxed);

                if volume != old {
                    let mut command = Command::new("osascript");

                    command.arg("-e").arg(format!("set Volume {}", volume));

                    old = volume;

                    command.output().unwrap();
                }

                // drop(new_data);

                thread::sleep(Duration::from_millis(100))
            }
        });

        Self { volume }
    }

    pub fn set(&self, volume: f32) {
        self.volume.swap(volume, Ordering::Relaxed);
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::new(5.0)
    }
}
