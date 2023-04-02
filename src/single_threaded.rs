use std::{time::SystemTime, marker::PhantomData, cell::Cell};
use super::*;

#[derive(Clone, Copy, Debug)]
pub struct LogObject<'a> {
    pub channel_id: Option<usize>,
    pub message: &'a str,
    pub severity: Level,
    pub time: SystemTime,
}

impl LogObject<'_> {
	pub fn new<'a>(channel_id: Option<usize>, severity: Level, message: &'a str) -> LogObject<'a> {
		LogObject { channel_id, message, severity, time: SystemTime::now() }
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

#[derive(Clone, Copy, Default)]
pub struct SimpleLogger<S: Sink> {
	sink: S,
	_unsync: PhantomData<Cell<()>>,
}

pub struct ChannelLogger<'a, S: Sink> {
	channel_id: usize,
	sink: &'a S,
	_unsendsync: PhantomData<*const ()>,
}

impl<S: Sink> Copy for ChannelLogger<'_, S> {}
impl<S: Sink> Clone for ChannelLogger<'_, S> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<S: Sink> SimpleLogger<S> {
	pub const fn new(sink: S) -> Self {
		Self { sink, _unsync: PhantomData }
	}

	pub const fn channel(&self, channel_id: usize) -> ChannelLogger<S> {
		ChannelLogger { channel_id, sink: &self.sink, _unsendsync: PhantomData }
	}

	pub fn sink(&self) -> &mut S {
		let ptr: *const S = &self.sink;
		let ptr = ptr as *mut S;
		unsafe { &mut *ptr }
	}
}

impl<S: Sink> ChannelLogger<'_, S> {
	pub const fn id(&self) -> usize {
		self.channel_id
	}

	pub fn sink(&self) -> &mut S {
		let ptr: *const S = self.sink;
		let ptr = ptr as *mut S;
		unsafe { &mut *ptr }
	}
}

impl<S: Sink> Logger for SimpleLogger<S> {
	fn log(&self, severity: Level, message: &str) {
		self.sink().consume(LogObject::new(None, severity, message))
	}
}

impl<S: Sink> Logger for ChannelLogger<'_, S> {
	fn log(&self, severity: Level, message: &str) {
		self.sink().consume(LogObject::new(Some(self.channel_id), severity, message))
	}
}
