use std::{
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub struct Volume {
    volume: Arc<Mutex<f32>>,
}

impl Volume {
    pub fn new(volume: f32) -> Self {
        let volume = Arc::new(Mutex::new(volume));

        let volume2 = volume.clone();

        thread::spawn(move || {
            let mut old = 11f32;
            loop {
                let new_data = volume2.lock().unwrap();

                if *new_data != old {
                    Command::new("osascript")
                        .arg("-e")
                        .arg(format!("set Volume {}", new_data))
                        .output()
                        .unwrap();

                    old = *new_data;
                }

                drop(new_data);

                thread::sleep(Duration::from_secs(1))
            }
        });

        Self { volume }
    }

    pub fn set(&self, volume: f32) {
        *self.volume.lock().unwrap() = volume
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::new(5.0)
    }
}
