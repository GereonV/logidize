//! Sensible [Logger]s.

pub mod single_threaded;
pub mod multi_threaded;

use std::{fmt::Display, thread::{self, ThreadId}, time::SystemTime};
#[doc(no_inline)]
pub use std::fmt::Arguments;

/// A logging severity level.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// Severity level `DEBUG`.
    DEBUG,
    /// Severity level `INFO`.
    INFO,
    /// Severity level `WARNING`.
    WARNING,
    /// Severity level `ERROR`.
    ERROR,
    /// Severity level `CRITICAL`.
    CRITICAL,
}

impl Level {
    /// Obtain the textual representation of the level.
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

/// A log-message with metadata.
///
/// Used by [single_threaded::SimpleLogger], [single_threaded::ChannelLogger], [multi_threaded::SimpleLogger], [multi_threaded::ChannelLogger].
#[derive(Clone, Copy, Debug)]
pub struct LogObject<'a> {
    /// The ID of the channel the log-request originated from.
    ///
    /// The main-channel (implicitly used by `SimpleLogger`s) has ID `0`.
    pub channel_id: usize,

    /// The log-message supplied by a call to [Logger::log()] or its family.
    pub message: Arguments<'a>,

    /// The severity level used to log this message.
    pub severity: Level,

    /// The [ThreadId] of the logging thread.
    pub thread_id: ThreadId,

    /// [SystemTime::now()] when this [LogObject] was created.
    pub time: SystemTime,
}

impl LogObject<'_> {
    /// Constructs a new [LogObject] with information about call-time calling thread.
    ///
    /// ```
    /// # use std::{thread, time::SystemTime};
    /// # use logidize::loggers::{Level, LogObject};
    /// let log_object = LogObject::new(0, Level::DEBUG, format_args!("test"));
    /// assert_eq!(log_object.thread_id, thread::current().id());
    /// assert_eq!(SystemTime::now().duration_since(log_object.time).unwrap().as_secs(), 0);
    /// ```
    pub fn new<'a>(channel_id: usize, severity: Level, message: Arguments<'a>) -> LogObject<'a> {
        LogObject {
            channel_id,
            message,
            severity,
            thread_id: thread::current().id(),
            time: SystemTime::now(),
        }
    }
}

/// A trait for objects which are capable of logging [Arguments] with a severity [Level].
pub trait Logger {
    /// Logs [Arguments] with severity [Level].
    fn log(&self, severity: Level, message: Arguments);
    /// Logs [Arguments] with severity [Level::DEBUG].
    fn    debug(&self, message: Arguments) { self.log(Level::DEBUG,    message); }
    /// Logs [Arguments] with severity [Level::INFO].
    fn     info(&self, message: Arguments) { self.log(Level::INFO,     message); }
    /// Logs [Arguments] with severity [Level::WARNING].
    fn  warning(&self, message: Arguments) { self.log(Level::WARNING,  message); }
    /// Logs [Arguments] with severity [Level::ERROR].
    fn    error(&self, message: Arguments) { self.log(Level::ERROR,    message); }
    /// Logs [Arguments] with severity [Level::CRITICAL].
    fn critical(&self, message: Arguments) { self.log(Level::CRITICAL, message); }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct MultiLogger<T1: Logger, T2: Logger>(pub T1, pub T2);

macro_rules! impl_levels {
	($($lvl:ident),*) => {
		$(
			fn $lvl(&self, message: Arguments) {
				self.0.$lvl(message);
				self.1.$lvl(message);
			}
		)*
	};
}

impl<T1: Logger, T2: Logger> Logger for MultiLogger<T1, T2> {
	fn log(&self, severity: Level, message: Arguments) {
		self.0.log(severity, message);
		self.1.log(severity, message);
	}

	impl_levels!(debug, info, warning, error, critical);
}

/// Creates a `MultiLogger` with the given given logger expressions.
#[macro_export]
macro_rules! multi_logger {
    ($head:expr, $tail:expr $(,)?) => {
        $crate::loggers::MultiLogger($head, $tail)
    };

    ($head:expr, $($tail:expr),+ $(,)?) => {
        $crate::loggers::MultiLogger($head, multi_sink!($($tail),+))
    };
}
