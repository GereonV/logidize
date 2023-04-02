use std::sync::{Mutex, LockResult, MutexGuard};
use super::*;

#[derive(Default)]
pub struct SimpleLogger<S: Sink> {
	sink: Mutex<S>,
}

impl<S: Sink + Clone> Clone for SimpleLogger<S> {
	fn clone(&self) -> Self {
		Self { sink: Mutex::new(self.sink.lock().expect("SimpleLogger::clone() failed because the logger was poisoned").clone()) }
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
		self.sink().expect("SimpleLogger::log() failed because the logger was poisoned").consume(LogObject::new(0, severity, message))
	}
}

impl<S: Sink> Logger for ChannelLogger<'_, S> {
	fn log(&self, severity: Level, message: &str) {
		self.sink().expect("ChannelLogger::log() failed because the underlying logger was poisoned").consume(LogObject::new(self.id, severity, message))
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let mut counter = 0;
        let logger = SimpleLogger::new(|_log_object: LogObject| {
            counter += 1;
        });
        std::thread::scope(|scope| {
            for _ in 0..10 {
                scope.spawn(|| {
                    for _ in 0..100_000 {
                        logger.debug("message");
                    }
                });
            }
        });
        assert_eq!(counter, 10 * 100_000);
    }

    #[test]
    fn test_channels() {
        let mut counters = [0; 10];
        let logger = SimpleLogger::new(|log_object: LogObject| {
            counters[log_object.channel_id] += 1;
        });
        std::thread::scope(|scope| {
            for i in 1..10 {
                let channel = logger.channel(i);
                scope.spawn(move || {
                    for _ in 0..(i * 100_000) {
                        channel.debug("message");
                    }
                });
            }
            for _ in 0..1_000_000 {
                logger.debug("message");
            }
        });
        assert_eq!(
            counters,
            [1_000_000, 100_000, 200_000, 300_000, 400_000, 500_000, 600_000, 700_000, 800_000, 900_000],
        );
    }
}
