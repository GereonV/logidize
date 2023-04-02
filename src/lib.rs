use std::{thread::{self, ThreadId}, time::SystemTime, fmt::Display};

pub mod single_threaded;
pub mod multi_threaded;
pub mod utils;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        write!(f, "{}", self.as_str())
    }
}

pub trait Logger {
    fn log(&self, severity: Level, message: &str);
    fn debug(&self, message: &str) { self.log(Level::DEBUG, message); }
    fn info(&self, message: &str) { self.log(Level::INFO, message); }
    fn warning(&self, message: &str) { self.log(Level::WARNING, message); }
    fn error(&self, message: &str) { self.log(Level::ERROR, message); }
    fn critical(&self, message: &str) { self.log(Level::CRITICAL, message); }
}

#[derive(Clone, Copy, Debug)]
pub struct LogObject<'a> {
    pub channel_id: usize, // 0 = no channel
    pub message: &'a str,
    pub severity: Level,
    pub thread_id: ThreadId,
    pub time: SystemTime,
}

impl LogObject<'_> {
	fn new<'a>(channel_id: usize, severity: Level, message: &'a str) -> LogObject<'a> {
		LogObject {
			channel_id,
			message,
			severity,
			thread_id: thread::current().id(),
			time: SystemTime::now(),
		}
	}
}

pub trait Sink {
	fn consume(&mut self, log_object: LogObject);
}

impl<T: FnMut(LogObject)> Sink for T {
	fn consume(&mut self, log_object: LogObject) {
		self(log_object);
	}
}
