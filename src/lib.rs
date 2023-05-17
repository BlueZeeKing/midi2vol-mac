use std::{
    ops::Add,
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use midi::{CCPacket, Connection, Error, PacketReceiver};
use vol::Volume;

pub mod midi;
pub mod vol;

pub struct MIDI2Vol {
    sender: Sender<Command>,
    thread: JoinHandle<()>,
}

pub enum Command {
    Packet(Result<CCPacket, midi::Error>),
    SetSleepTime(Duration),
    SetChannel(u8),
    SetCCNum(u8),
    SetSourceIndex(usize),
    Start,
    Stop,
    GetState(oneshot::Sender<Settings>),
    GetStatus(oneshot::Sender<Option<Error>>),
}

pub struct Settings {
    pub channel: u8,
    pub cc_num: u8,
    pub vol_sample_time: Duration,
    pub source_index: usize,
}

impl MIDI2Vol {
    pub fn new() -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel::<Command>();

        let tx1 = tx.clone();

        let thread = thread::spawn(move || {
            let volume = Volume::new(0.0, Duration::from_millis(100));
            let (mut connection, receiver) = Connection::new(0).unwrap();

            let mut cc_num: u8 = 0x3E;
            let mut channel: u8 = 1;

            let mut error: Option<Error> = None;

            spawn_thread(receiver, tx1.clone());

            for command in rx {
                match command {
                    Command::Packet(packet) => {
                        error = packet.clone().err();
                        if let Ok(packet) = packet {
                            if packet.cc_num == cc_num && packet.channel == channel {
                                volume.set((packet.val as f32 / 127.0 * 70.0).round() / 10.0)
                            }
                        }
                    }
                    Command::SetSleepTime(duration) => volume.set_sleep_time(duration),
                    Command::SetChannel(new_channel) => channel = new_channel,
                    Command::SetCCNum(new_cc_num) => cc_num = new_cc_num,
                    Command::SetSourceIndex(index) => {
                        spawn_thread(connection.set_source_index(index), tx1.clone())
                    }
                    Command::Start => spawn_thread(connection.start(), tx1.clone()),
                    Command::Stop => connection.stop(),
                    Command::GetState(sender) => sender
                        .send(Settings {
                            channel,
                            cc_num,
                            vol_sample_time: volume.get_sleep_time(),
                            source_index: connection.get_source_index(),
                        })
                        .unwrap(),
                    Command::GetStatus(sender) => sender.send(error.clone()).unwrap(),
                }
            }
        });

        Ok(Self { sender: tx, thread })
    }

    pub fn set_sleep_time(&self, time: Duration) -> Result<(), ChannelError> {
        Ok(self.sender.send(Command::SetSleepTime(time))?)
    }

    pub fn set_channel(&self, channel: u8) -> Result<(), ChannelError> {
        Ok(self.sender.send(Command::SetChannel(channel))?)
    }

    pub fn set_cc_num(&self, cc_num: u8) -> Result<(), ChannelError> {
        Ok(self.sender.send(Command::SetCCNum(cc_num))?)
    }

    pub fn set_source_index(&self, index: usize) -> Result<(), ChannelError> {
        Ok(self.sender.send(Command::SetSourceIndex(index))?)
    }

    pub fn start(&self) -> Result<(), ChannelError> {
        Ok(self.sender.send(Command::Start)?)
    }

    pub fn stop(&self) -> Result<(), ChannelError> {
        Ok(self.sender.send(Command::Stop)?)
    }

    pub fn get_settings(&self) -> Result<Settings, ChannelError> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(Command::GetState(tx))?;
        Ok(rx.recv_deadline(Instant::now().add(Duration::from_secs(5)))?)
    }

    pub fn get_error(&self) -> Result<Option<Error>, ChannelError> {
        if self.thread.is_finished() {
            return Ok(Some(Error::ConnectionThreadFailure));
        }

        let (tx, rx) = oneshot::channel();
        self.sender.send(Command::GetStatus(tx))?;
        Ok(rx.recv_deadline(Instant::now().add(Duration::from_secs(5)))?)
    }
}

#[derive(Debug)]
pub enum ChannelError {
    Send(mpsc::SendError<Command>),
    Receive(oneshot::RecvTimeoutError),
}

impl From<mpsc::SendError<Command>> for ChannelError {
    fn from(value: mpsc::SendError<Command>) -> Self {
        Self::Send(value)
    }
}

impl From<oneshot::RecvTimeoutError> for ChannelError {
    fn from(value: oneshot::RecvTimeoutError) -> Self {
        Self::Receive(value)
    }
}

fn spawn_thread(receiver: PacketReceiver, tx1: Sender<Command>) {
    thread::spawn(move || {
        for packet in receiver.iter() {
            tx1.send(Command::Packet(packet)).unwrap();
        }
    });
}
