use std::{io::Write};
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

	fn map(&mut self, channel_id: usize) -> Option<Self::DisplayType>;
}

impl<T: FnMut(usize) -> Option<DisplayType>, DisplayType: Display> ChannelMap for T {
	type DisplayType = DisplayType;

	fn map(&mut self, channel_id: usize) -> Option<Self::DisplayType> {
		self(channel_id)
	}
}

pub struct StdErrSink<M: ChannelMap> {
	channel_map: M,
	min_severity: Level,
	mute: bool,
}

impl<M: ChannelMap> StdErrSink<M> {
	pub const fn new(channel_map: M) -> Self {
		Self {
			channel_map,
			min_severity: Level::DEBUG,
			mute: false,
		}
	}

	pub const fn channel_map(&self) -> &M {
		&self.channel_map
	}

	pub fn channel_map_mut(&mut self) -> &mut M {
		&mut self.channel_map
	}

	pub const fn min_severity(&self) -> Level {
		self.min_severity
	}

	pub fn set_min_severity(&mut self, min_severity: Level) {
		self.min_severity = min_severity;
	}

	pub const fn muted(&self) -> bool {
		self.mute
	}

	pub fn mute(&mut self) {
		self.mute = true;
	}

	pub fn unmute(&mut self) {
		self.mute = false;
	}
}

impl<M: ChannelMap + Default> Default for StdErrSink<M> {
	fn default() -> Self {
		Self::new(Default::default())
	}
}

impl<M: ChannelMap> Sink for StdErrSink<M> {
	fn consume(&mut self, log_object: LogObject) {
		if self.mute || log_object.severity < self.min_severity {
			return;
		}
		let Some(channel_name) = self.channel_map.map(log_object.channel_id) else {
			return;
		};
		let level = match log_object.severity {
			Level::DEBUG    => concatcp!(SET_COLOR_BRIGHT_CYAN   , Level::DEBUG.as_str()   , RESET_COLOR),
			Level::INFO     => concatcp!(SET_COLOR_BRIGHT_BLUE   , Level::INFO.as_str()    , RESET_COLOR),
			Level::WARNING  => concatcp!(SET_COLOR_BRIGHT_YELLOW , Level::WARNING.as_str() , RESET_COLOR),
			Level::ERROR    => concatcp!(SET_COLOR_BRIGHT_RED    , Level::ERROR.as_str()   , RESET_COLOR),
			Level::CRITICAL => concatcp!(SET_COLOR_BRIGHT_MAGENTA, Level::CRITICAL.as_str(), RESET_COLOR),
		};
		// TODO add timestamp
		let _ = writeln!(std::io::stderr(), "[{}][{}]: {}", level, channel_name, log_object.message);
	}
}
