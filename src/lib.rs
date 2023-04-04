pub mod colors;
pub mod filter_maps;
pub mod loggers;
pub mod sinks;
pub mod writers;

use crate::{
	filter_maps::SimpleChannelFilterMap,
	loggers::multi_threaded::SimpleLogger,
	sinks::WriteSink,
	writers::StderrWriter,
};

// Default::default() is not const
pub static GLOBAL_LOGGER: SimpleLogger<WriteSink<StderrWriter, SimpleChannelFilterMap<String>>> = SimpleLogger::new(
    WriteSink::new(StderrWriter, SimpleChannelFilterMap::new())
);

#[macro_export]
macro_rules! set_default_logger {
	($($logger:tt)*) => {
		#[macro_export]
		macro_rules! default_logger {
			() => {
				$($logger)*
			};
		}
	};
}

set_default_logger!($crate::GLOBAL_LOGGER);

#[macro_export]
macro_rules! log {
    ($lvl:expr, $fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::log!(default_logger!(), $lvl, $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $lvl:expr, $($args:tt),+$(,)?) => {
        $logger.log($lvl, format_args!($($args),+))
    };
}

#[macro_export]
macro_rules! debug {
    ($fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::debug!(default_logger!(), $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $($args:tt),+$(,)?) => {
        $logger.debug(format_args!($($args),+))
    };
}

#[macro_export]
macro_rules! info {
    ($fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::info!(default_logger!(), $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $($args:tt),+$(,)?) => {
        $logger.info(format_args!($($args),+))
    };
}

#[macro_export]
macro_rules! warning {
    ($fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::warning!(default_logger!(), $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $($args:tt),+$(,)?) => {
        $logger.warning(format_args!($($args),+))
    };
}

#[macro_export]
macro_rules! error {
    ($fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::error!(default_logger!(), $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $($args:tt),+$(,)?) => {
        $logger.error(format_args!($($args),+))
    };
}

#[macro_export]
macro_rules! critical {
    ($fmt:literal$(, $($($args:tt),+$(,)?)?)?) => {
        $crate::critical!(default_logger!(), $fmt, $($($($args),+)?)?)
    };

    ($logger:expr, $($args:tt),+$(,)?) => {
        $logger.critical(format_args!($($args),+))
    };
}
