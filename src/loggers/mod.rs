pub mod single_threaded;
pub mod multi_threaded;

use std::{fmt::Display, thread::{self, ThreadId}, time::SystemTime};
pub use std::fmt::Arguments;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    DEBUG,
    INFO,
    WARNING,
    ERROR,
    CRITICAL,
}

impl Level {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Level::DEBUG    => "DEBUG",
            Level::INFO     => "INFO",
            Level::WARNING  => "WARNING",
            Level::ERROR    => "ERROR",
            Level::CRITICAL => "CRITICAL",
        }
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LogObject<'a> {
    pub channel_id: usize, // 0 = no channel
    pub message: Arguments<'a>,
    pub severity: Level,
    pub thread_id: ThreadId,
    pub time: SystemTime,
}

impl LogObject<'_> {
    fn new<'a>(channel_id: usize, severity: Level, message: Arguments<'a>) -> LogObject<'a> {
        LogObject {
            channel_id,
            message,
            severity,
            thread_id: thread::current().id(),
            time: SystemTime::now(),
        }
    }
}

pub trait Logger {
    fn log(&self, severity: Level, message: Arguments);
    fn    debug(&self, message: Arguments) { self.log(Level::DEBUG,    message); }
    fn     info(&self, message: Arguments) { self.log(Level::INFO,     message); }
    fn  warning(&self, message: Arguments) { self.log(Level::WARNING,  message); }
    fn    error(&self, message: Arguments) { self.log(Level::ERROR,    message); }
    fn critical(&self, message: Arguments) { self.log(Level::CRITICAL, message); }
}
