//! A lightweight and performant logging utility

#![warn(missing_docs)]

pub mod colors;
pub mod filter_maps;
pub mod loggers;
pub mod sinks;
pub mod writers;

#[doc(hidden)]
pub use loggers::Logger;

use crate::{
	filter_maps::SimpleChannelFilterMap,
	loggers::multi_threaded::SimpleLogger,
	sinks::WriteSink,
	writers::StderrWriter,
};

// Default::default() is not const
/// A sensible default logger for use in multithreaded applications.
pub static GLOBAL_LOGGER: SimpleLogger<WriteSink<StderrWriter, SimpleChannelFilterMap<String>>> = SimpleLogger::new(
    WriteSink::new(StderrWriter, SimpleChannelFilterMap::new())
);

/// Invoked to retrieve a default [Logger](loggers::Logger) in logging-macros like [log!].
#[macro_export]
macro_rules! default_logger {
    () => {
        $crate::GLOBAL_LOGGER
    };
}

/// Invokes [Logger::log()](loggers::Logger::log()) using [format_args!].
/// 
/// Defaults to using [default_logger!].
#[macro_export]
macro_rules! log {
    ($lvl:expr, $fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::log!(default_logger!(), $lvl, $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $lvl:expr, $($args:tt),+$(,)?) => {
        $logger.log($lvl, format_args!($($args),+))
    };
}

/// Invokes [Logger::debug()](loggers::Logger::debug()) using [format_args!].
/// 
/// Defaults to using [default_logger!].
#[macro_export]
macro_rules! debug {
    ($fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::debug!(default_logger!(), $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $($args:tt),+$(,)?) => {
        $logger.debug(format_args!($($args),+))
    };
}


/// Invokes [Logger::info()](loggers::Logger::info()) using [format_args!].
/// 
/// Defaults to using [default_logger!].
#[macro_export]
macro_rules! info {
    ($fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::info!(default_logger!(), $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $($args:tt),+$(,)?) => {
        $logger.info(format_args!($($args),+))
    };
}

/// Invokes [Logger::warning()](loggers::Logger::warning()) using [format_args!].
/// 
/// Defaults to using [default_logger!].
#[macro_export]
macro_rules! warning {
    ($fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::warning!(default_logger!(), $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $($args:tt),+$(,)?) => {
        $logger.warning(format_args!($($args),+))
    };
}

/// Invokes [Logger::error()](loggers::Logger::error()) using [format_args!].
/// 
/// Defaults to using [default_logger!].
#[macro_export]
macro_rules! error {
    ($fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::error!(default_logger!(), $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $($args:tt),+$(,)?) => {
        $logger.error(format_args!($($args),+))
    };
}

/// Invokes [Logger::critical()](loggers::Logger::critical()) using [format_args!].
/// 
/// Defaults to using [default_logger!].
#[macro_export]
macro_rules! critical {
    ($fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::critical!(default_logger!(), $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $($args:tt),+$(,)?) => {
        $logger.critical(format_args!($($args),+))
    };
}
