use std::{thread::{self, ThreadId}, time::SystemTime, fmt::{Display,  Arguments}};

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
    fn log(&self, severity: Level, message: Arguments);
    fn debug(&self, message: Arguments) { self.log(Level::DEBUG, message); }
    fn info(&self, message: Arguments) { self.log(Level::INFO, message); }
    fn warning(&self, message: Arguments) { self.log(Level::WARNING, message); }
    fn error(&self, message: Arguments) { self.log(Level::ERROR, message); }
    fn critical(&self, message: Arguments) { self.log(Level::CRITICAL, message); }
}

#[macro_export]
macro_rules! log {
    ($logger:expr, $lvl:expr, $($args:tt),+) => {
        $logger.log($lvl, format_args!($($args),+))
    };
}

#[macro_export]
macro_rules! debug {
    ($logger:expr, $($args:tt),+) => {
        $logger.debug(format_args!($($args),+))
    };
}

#[macro_export]
macro_rules! info {
    ($logger:expr, $($args:tt),+) => {
        $logger.info(format_args!($($args),+))
    };
}

#[macro_export]
macro_rules! warning {
    ($logger:expr, $($args:tt),+) => {
        $logger.warning(format_args!($($args),+))
    };
}

#[macro_export]
macro_rules! error {
    ($logger:expr, $($args:tt),+) => {
        $logger.error(format_args!($($args),+))
    };
}

#[macro_export]
macro_rules! critical {
    ($logger:expr, $($args:tt),+) => {
        $logger.critical(format_args!($($args),+))
    };
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

pub trait Sink {
	fn consume(&mut self, log_object: LogObject);
}

impl<T: FnMut(LogObject)> Sink for T {
	fn consume(&mut self, log_object: LogObject) {
		self(log_object);
	}
}
