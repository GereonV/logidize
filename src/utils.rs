use std::{io::Write, time::UNIX_EPOCH};
use const_format::concatcp;

use super::*;

pub const SET_COLOR_BRIGHT_RED     : &str = "\x1b[1;31m";
pub const SET_COLOR_BRIGHT_GREEN   : &str = "\x1b[1;32m";
pub const SET_COLOR_BRIGHT_YELLOW  : &str = "\x1b[1;33m";
pub const SET_COLOR_BRIGHT_BLUE    : &str = "\x1b[1;34m";
pub const SET_COLOR_BRIGHT_MAGENTA : &str = "\x1b[1;35m";
pub const SET_COLOR_BRIGHT_CYAN    : &str = "\x1b[1;36m";
pub const SET_COLOR_BRIGHT_WHITE   : &str = "\x1b[1;37m";
pub const SET_COLOR_DEFAULT        : &str = "\x1b[39m";
pub const RESET_COLOR              : &str = "\x1b[0m";

#[derive(Clone, Copy, Debug, Default)]
pub struct StderrWrite;

impl Write for StderrWrite {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		std::io::stderr().write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		std::io::stderr().flush()
	}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdoutWrite;

impl Write for StdoutWrite {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		std::io::stdout().write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		std::io::stdout().flush()
	}
}

pub trait ChannelFilterMap {
	type DisplayType: Display;

	fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType>;
}

impl<T: FnMut(&LogObject) -> Option<DisplayType>, DisplayType: Display> ChannelFilterMap for T {
	type DisplayType = DisplayType;

	fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
		self(log_object)
	}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InvisibleChannelFilterMap;
impl ChannelFilterMap for InvisibleChannelFilterMap {
	type DisplayType = usize;

	fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
		Some(log_object.channel_id)
	}
}

pub struct WriteSink<W: Write = StderrWrite, M: ChannelFilterMap = InvisibleChannelFilterMap> {
	pub channel_map: M,
	pub colors: bool,
	pub log_thread_id: bool,
	pub min_severity: Level,
	pub muted: bool,
	pub output: W,
}

impl<W: Write, M: ChannelFilterMap> WriteSink<W, M> {
	pub const fn new(output: W, channel_map: M) -> Self {
		Self {
			channel_map,
			colors: true,
			log_thread_id: false,
			min_severity: Level::DEBUG,
			muted: false,
			output
		}
	}
}

impl<W: Write + Default, M: ChannelFilterMap + Default> Default for WriteSink<W, M> {
	fn default() -> Self {
		Self::new(Default::default(), Default::default())
	}
}

impl<W: Write, M: ChannelFilterMap> Sink for WriteSink<W, M> {
	fn consume(&mut self, log_object: LogObject) {
		if self.muted || log_object.severity < self.min_severity {
			return;
		}
		let Some(channel_name) = self.channel_map.filter_map(&log_object) else {
			return;
		};
		let secs_since_epoch = match log_object.time.duration_since(UNIX_EPOCH) {
			Ok(duration) => duration.as_secs() as i64,
			Err(e) => -(e.duration().as_secs() as i64),
		};
		let id: u64 = unsafe { std::mem::transmute(log_object.thread_id) };
		let _ = if self.colors {
			let level = match log_object.severity {
				Level::DEBUG    => concatcp!(SET_COLOR_BRIGHT_CYAN   , Level::DEBUG.as_str()),
				Level::INFO     => concatcp!(SET_COLOR_BRIGHT_BLUE   , Level::INFO.as_str()),
				Level::WARNING  => concatcp!(SET_COLOR_BRIGHT_YELLOW , Level::WARNING.as_str()),
				Level::ERROR    => concatcp!(SET_COLOR_BRIGHT_RED    , Level::ERROR.as_str()),
				Level::CRITICAL => concatcp!(SET_COLOR_BRIGHT_MAGENTA, Level::CRITICAL.as_str()),
			};
			if self.log_thread_id {
				writeln!(self.output, "[{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{secs_since_epoch}{RESET_COLOR}][{level}{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}{channel_name}{RESET_COLOR}]: {}", log_object.message)
			} else {
				writeln!(self.output, "[{SET_COLOR_BRIGHT_GREEN}{secs_since_epoch}{RESET_COLOR}][{level}{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}{channel_name}{RESET_COLOR}]: {}", log_object.message)
			}
		} else if self.log_thread_id {
			writeln!(self.output, "[{id}][{secs_since_epoch}][{}][{channel_name}]: {}", log_object.severity.as_str(), log_object.message)
		} else {
			writeln!(self.output, "[{secs_since_epoch}][{}][{channel_name}]: {}", log_object.severity.as_str(), log_object.message)
		};
	}
}

#[cfg(test)]
mod tests {
	use crate::single_threaded::SimpleLogger;
	use super::*;

	#[test]
	fn test_basics() {
		let mut output = Vec::<u8>::new();
		let logger = SimpleLogger::new(WriteSink::new(&mut output, InvisibleChannelFilterMap));
		debug!(logger, "debug");
		info!(logger, "info");
		warning!(logger, "warning");
		error!(logger, "error");
		critical!(logger, "critical");
		debug!(logger.channel(1), "from channel {}", 1);
		debug!(logger.channel(2), "from channel {}", 2);
		let mut iter = output.iter();
		let _ = iter.by_ref().find(|char| **char == b'm');
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..5]).unwrap(), "DEBUG");
		let _ = iter.by_ref().find(|char| **char == b'm');
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..12]).unwrap(), "][0]: debug\n");
		let _ = iter.by_ref().find(|char| **char == b'm');
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..4]).unwrap(), "INFO");
		let _ = iter.by_ref().find(|char| **char == b':');
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..6]).unwrap(), " info\n");
		let _ = iter.by_ref().filter(|char| **char == b'\n').take(4).last();
		let _ = iter.by_ref().filter(|char| **char == b']').take(2).last();
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..20]).unwrap(), "[1]: from channel 1\n");
	}
}
