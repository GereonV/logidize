use std::{thread, thread::ThreadId, time::SystemTime, sync::{Mutex, LockResult, MutexGuard}};
use super::*;

#[derive(Clone, Copy, Debug)]
pub struct LogObject<'a> {
    pub channel_id: Option<usize>,
    pub message: &'a str,
    pub severity: Level,
	pub thread_id: ThreadId,
    pub time: SystemTime,
}

impl LogObject<'_> {
	pub fn new<'a>(channel_id: Option<usize>, severity: Level, message: &'a str) -> LogObject<'a> {
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

#[derive(Default)]
pub struct SimpleLogger<S: Sink> {
	sink: Mutex<S>,
}

impl<S: Sink + Clone> Clone for SimpleLogger<S> {
	fn clone(&self) -> Self {
		Self { sink: Mutex::new(self.sink.lock().unwrap().clone()) }
	}
}

pub struct ChannelLogger<'a, S: Sink> {
	id: usize,
	sink: &'a Mutex<S>,
}

impl<S: Sink> Copy for ChannelLogger<'_, S> {}
impl<S: Sink> Clone for ChannelLogger<'_, S> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<S: Sink> SimpleLogger<S> {
	pub const fn new(sink: S) -> Self {
		Self { sink: Mutex::new(sink) }
	}

	pub const fn channel(&self, channel_id: usize) -> ChannelLogger<S> {
		ChannelLogger { id: channel_id, sink: &self.sink }
	}

	pub fn sink(&self) -> LockResult<MutexGuard<'_, S>> {
		self.sink.lock()
	}
}

impl<S: Sink> ChannelLogger<'_, S> {
	pub const fn id(&self) -> usize {
		self.id
	}

	pub fn sink(&self) -> LockResult<MutexGuard<'_, S>> {
		self.sink.lock()
	}
}

impl<S: Sink> Logger for SimpleLogger<S> {
	fn log(&self, severity: Level, message: &str) {
		self.sink().unwrap().consume(LogObject::new(None, severity, message))
	}
}

impl<S: Sink> Logger for ChannelLogger<'_, S> {
	fn log(&self, severity: Level, message: &str) {
		self.sink().unwrap().consume(LogObject::new(Some(self.id), severity, message))
	}
}
