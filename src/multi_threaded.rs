use std::sync::{Mutex, LockResult, MutexGuard};
use super::*;

#[derive(Debug, Default)]
pub struct SimpleLogger<S: Sink> {
    sink: Mutex<S>,
}

impl<S: Sink + Clone> Clone for SimpleLogger<S> {
    fn clone(&self) -> Self {
        Self { sink: Mutex::new(self.sink.lock().expect("SimpleLogger::clone() failed because the logger was poisoned").clone()) }
    }
}

#[derive(Debug)]
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

    pub fn into_sink(self) -> LockResult<S> {
        self.sink.into_inner()
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
    fn log(&self, severity: Level, message: Arguments) {
        self.sink().expect("SimpleLogger::log() failed because the logger was poisoned").consume(LogObject::new(0, severity, message))
    }
}

impl<S: Sink> Logger for ChannelLogger<'_, S> {
    fn log(&self, severity: Level, message: Arguments) {
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
                        debug!(logger, "message");
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
            for _ in 0..10 {
                scope.spawn(|| {
                    for i in 0..10 {
                        let channel = logger.channel(i);
                        for _ in 0..((i + 1) * 1_000) {
                            debug!(channel, "message");
                        }
                    }
                });
            }
        });
        assert_eq!(
            counters,
            [10_000, 20_000, 30_000, 40_000, 50_000, 60_000, 70_000, 80_000, 90_000, 100_000],
        );
    }
}
