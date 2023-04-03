use std::{io::Write, time::UNIX_EPOCH, collections::{BTreeMap, btree_map::Entry}, ptr::NonNull};
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Colored(pub Level);

impl Colored {
	pub const fn as_str(&self) -> &'static str {
		match self.0 {
			Level::DEBUG    => concatcp!(SET_COLOR_BRIGHT_CYAN   , Level::DEBUG.as_str()   , RESET_COLOR),
			Level::INFO     => concatcp!(SET_COLOR_BRIGHT_BLUE   , Level::INFO.as_str()    , RESET_COLOR),
			Level::WARNING  => concatcp!(SET_COLOR_BRIGHT_YELLOW , Level::WARNING.as_str() , RESET_COLOR),
			Level::ERROR    => concatcp!(SET_COLOR_BRIGHT_RED    , Level::ERROR.as_str()   , RESET_COLOR),
			Level::CRITICAL => concatcp!(SET_COLOR_BRIGHT_MAGENTA, Level::CRITICAL.as_str(), RESET_COLOR),
		}
	}
}

impl Display for Colored {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}

#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct StderrWriter;

impl Write for StderrWriter {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		std::io::stderr().write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		std::io::stderr().flush()
	}
}

#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct StdoutWriter;

impl Write for StdoutWriter {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		std::io::stdout().write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		std::io::stdout().flush()
	}
}

#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct MultiWriter<T1: Write, T2: Write>(T1, T2);

impl<T1: Write, T2: Write> MultiWriter<T1, T2> {
	pub const fn new(t1: T1, t2: T2) -> Self {
		Self(t1, t2)
	}
}

impl<T1: Write, T2: Write> Write for MultiWriter<T1, T2> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.0.write(buf).and(self.1.write(buf))
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.0.flush().and(self.1.flush())
	}
}

#[macro_export]
macro_rules! multi_writer {
	($head:expr, $tail:expr $(,)?) => {
		MultiWriter::new($head, $tail)
	};

	($head:expr, $($tail:expr),+ $(,)?) => {
		MultiWriter::new($head, multi_writer!($($tail),+))
	};
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

#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct InvisibleChannelFilterMap;
impl ChannelFilterMap for InvisibleChannelFilterMap {
	type DisplayType = usize;

	fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
		Some(log_object.channel_id)
	}
}

// unsafe because it may outlive borrowed data
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BorrowDisplay<T: Display>(NonNull<T>);

impl<T: Display> Display for BorrowDisplay<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let t = unsafe { self.0.as_ref() };
		t.fmt(f)
	}
}

#[derive(Clone, Debug, Default)]
pub struct SimpleChannelFilterMap<T: Display> {
	channels: BTreeMap<usize, (T, bool)>, // (name, enabled)
}

impl<T: Display> SimpleChannelFilterMap<T> {
	pub const fn new() -> Self {
		Self { channels: BTreeMap::new() }
	}

	pub fn channel(&self, channel_id: usize) -> Option<&(T, bool)> {
		self.channels.get(&channel_id)
	}

	pub fn channel_mut(&mut self, channel_id: usize) -> Option<&mut (T, bool)> {
		self.channels.get_mut(&channel_id)
	}

	pub fn set_channel(&mut self, channel_id: usize, channel_name: impl Into<T>, enabled: bool) -> Option<(T, bool)> {
		self.channels.insert(channel_id, (channel_name.into(), enabled))
	}

	pub fn insert_channel(&mut self, channel_id: usize, channel_name: impl Into<T>, enabled: bool) -> (bool, &mut (T, bool)) {
		self.insert_channel_with_id(channel_id, |_| (channel_name.into(), enabled))
	}

	pub fn insert_channel_with(&mut self, channel_id: usize, f: impl FnOnce() -> (T, bool)) -> (bool, &mut (T, bool)) {
		self.insert_channel_with_id(channel_id, |_| f())
	}

	pub fn insert_channel_with_id(&mut self, channel_id: usize, f: impl FnOnce(usize) -> (T, bool)) -> (bool, &mut (T, bool)) {
		match self.channels.entry(channel_id) {
			Entry::Occupied(e) => (false, e.into_mut()),
			Entry::Vacant(e) => (true, e.insert(f(channel_id))),
		}
	}

	pub fn channel_name(&self, channel_id: usize) -> Option<&T> {
		let (name, _) = self.channels.get(&channel_id)?;
		Some(name)
	}

	pub fn set_channel_name(&mut self, channel_id: usize, channel_name: impl Into<T>) {
		match self.channels.get_mut(&channel_id) {
			Some(channel) => channel.0 = channel_name.into(),
			None => panic!("called SimpleChannelFilterMap::set_channel_name() with unknown channel"),
		}
	}

	pub fn channel_enabled(&self, channel_id: usize) -> bool {
		match self.channels.get(&channel_id) {
			Some((_, true)) => true,
			_ => false,
		}
	}

	pub fn set_channel_enabled(&mut self, channel_id: usize, enabled: bool) {
		match self.channels.get_mut(&channel_id) {
			Some(channel) => channel.1 = enabled,
			None => panic!("called SimpleChannelFilterMap::set_channel_enabled() with unknown channel"),
		}
	}
}

impl<T: Display> ChannelFilterMap for SimpleChannelFilterMap<T> {
	type DisplayType = BorrowDisplay<T>;

	fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
		let (name, enabled) = self.channels.get(&log_object.channel_id)?;
		if !enabled {
			return None;
		}
		let ptr = name as *const T;
		let ptr = unsafe {
			NonNull::new_unchecked(ptr as *mut T)
		};
		Some(BorrowDisplay(ptr))
	}
}

#[derive(Clone, Copy, Debug)]
pub struct WriteSink<W: Write = StderrWriter, M: ChannelFilterMap = InvisibleChannelFilterMap> {
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
			let level = Colored(log_object.severity).as_str();
			if self.log_thread_id {
				writeln!(self.output, "[{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{secs_since_epoch}{RESET_COLOR}][{level}][{SET_COLOR_BRIGHT_WHITE}{channel_name}{RESET_COLOR}]: {}", log_object.message)
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
		let _ = iter.by_ref().filter(|char| **char == b'm').take(3).last();
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..5]).unwrap(), "DEBUG");
		let _ = iter.by_ref().filter(|char| **char == b'm').take(2).last();
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..1]).unwrap(), "0");
		let _ = iter.by_ref().filter(|char| **char == b'm').take(1).last();
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..9]).unwrap(), "]: debug\n");

		let _ = iter.by_ref().filter(|char| **char == b'm').take(3).last();
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..4]).unwrap(), "INFO");
		let _ = iter.by_ref().filter(|char| **char == b'm').take(2).last();
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..1]).unwrap(), "0");
		let _ = iter.by_ref().filter(|char| **char == b'm').take(1).last();
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..8]).unwrap(), "]: info\n");

		let _ = iter.by_ref().filter(|char| **char == b'\n').take(4).last();
		let _ = iter.by_ref().filter(|char| **char == b'm').take(5).last();
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..1]).unwrap(), "1");
		let _ = iter.by_ref().filter(|char| **char == b'm').take(1).last();
		assert_eq!(std::str::from_utf8(&iter.as_slice()[..18]).unwrap(), "]: from channel 1\n");
	}
}
