use std::{marker::PhantomData, cell::Cell};
use super::*;

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

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;

    #[test]
    fn test_simple() {
        let start_time = SystemTime::now();
        let logger = SimpleLogger::new(|log_object: LogObject| {
            assert!(log_object.time >= start_time);
            assert!(log_object.time <= SystemTime::now());
            assert_eq!(log_object.channel_id, None);
            assert_eq!(log_object.severity, Level::DEBUG);
            assert_eq!(log_object.message, "message");
        });
        logger.debug("message");
        logger.log(Level::DEBUG, "message");
    }

    #[test]
    fn test_channels() {
        let mut counter = 0;
        let logger = SimpleLogger::new(|log_object: LogObject| {
            assert_eq!(log_object.channel_id, Some(counter));
            counter += 1;
        });
        for i in 0..10 {
            let channel = logger.channel(i);
            channel.debug("message");
        }
    }
}
