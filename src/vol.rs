use std::{
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use atomic_float::AtomicF32;

#[derive(Clone)]
pub struct Volume {
    volume: Arc<AtomicF32>,
    sleep_time: Arc<AtomicU64>,
}

impl Volume {
    pub fn new(volume: f32, sleep_time: Duration) -> Self {
        let volume = Arc::new(AtomicF32::new(volume));
        let sleep_time = Arc::new(AtomicU64::new(sleep_time.as_millis() as u64));

        let volume2 = volume.clone();
        let sleep_time2 = sleep_time.clone();

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

                thread::sleep(Duration::from_millis(sleep_time2.load(Ordering::Relaxed)))
            }
        });

        Self { volume, sleep_time }
    }

    pub fn set(&self, volume: f32) {
        self.volume.swap(volume, Ordering::Relaxed);
    }

    pub fn set_sleep_time(&self, sleep_time: Duration) {
        self.sleep_time
            .swap(sleep_time.as_millis() as u64, Ordering::Relaxed);
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::new(5.0, Duration::from_millis(100))
    }
}
