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

pub trait ChannelMap {
	type DisplayType: Display;

	fn map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType>;
}

impl<T: FnMut(&LogObject) -> Option<DisplayType>, DisplayType: Display> ChannelMap for T {
	type DisplayType = DisplayType;

	fn map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
		self(log_object)
	}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InvisibleChannelMap;
impl ChannelMap for InvisibleChannelMap {
	type DisplayType = usize;

	fn map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
		Some(log_object.channel_id)
	}
}

pub struct StdErrSink<M: ChannelMap = InvisibleChannelMap> {
	pub channel_map: M,
	pub log_thread_id: bool,
	pub min_severity: Level,
	pub muted: bool,
}

impl<M: ChannelMap> StdErrSink<M> {
	pub const fn new(channel_map: M) -> Self {
		Self {
			channel_map,
			log_thread_id: false,
			min_severity: Level::DEBUG,
			muted: false,
		}
	}
}

impl<M: ChannelMap + Default> Default for StdErrSink<M> {
	fn default() -> Self {
		Self::new(Default::default())
	}
}

impl<M: ChannelMap> Sink for StdErrSink<M> {
	fn consume(&mut self, log_object: LogObject) {
		if self.muted || log_object.severity < self.min_severity {
			return;
		}
		let Some(channel_name) = self.channel_map.map(&log_object) else {
			return;
		};
		let level = match log_object.severity {
			Level::DEBUG    => concatcp!(SET_COLOR_BRIGHT_CYAN   , Level::DEBUG.as_str()   , RESET_COLOR),
			Level::INFO     => concatcp!(SET_COLOR_BRIGHT_BLUE   , Level::INFO.as_str()    , RESET_COLOR),
			Level::WARNING  => concatcp!(SET_COLOR_BRIGHT_YELLOW , Level::WARNING.as_str() , RESET_COLOR),
			Level::ERROR    => concatcp!(SET_COLOR_BRIGHT_RED    , Level::ERROR.as_str()   , RESET_COLOR),
			Level::CRITICAL => concatcp!(SET_COLOR_BRIGHT_MAGENTA, Level::CRITICAL.as_str(), RESET_COLOR),
		};
		let secs_since_epoch = match log_object.time.duration_since(UNIX_EPOCH) {
			Ok(duration) => duration.as_secs() as i64,
			Err(e) => -(e.duration().as_secs() as i64),
		};
		let _ = if self.log_thread_id {
			let id: u64 = unsafe { std::mem::transmute(log_object.thread_id) };
			writeln!(std::io::stderr(), "[{id}][{secs_since_epoch}][{level}][{channel_name}]: {}", log_object.message)
		} else {
			writeln!(std::io::stderr(), "[{secs_since_epoch}][{level}][{channel_name}]: {}", log_object.message)
		};
	}
}
