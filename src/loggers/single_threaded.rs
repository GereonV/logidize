use std::{marker::PhantomData, cell::Cell};

use crate::{
    loggers::{Arguments, Level, Logger, LogObject},
    sinks::Sink,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct SimpleLogger<S: Sink> {
    sink: S,
    _unsync: PhantomData<Cell<()>>,
}

#[derive(Debug)]
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
    #[must_use]
    pub const fn new(sink: S) -> Self {
        Self { sink, _unsync: PhantomData }
    }

    #[must_use]
    pub const fn channel(&self, channel_id: usize) -> ChannelLogger<S> {
        ChannelLogger { channel_id, sink: &self.sink, _unsendsync: PhantomData }
    }

    // can't be called simultaneously due to thread-limitations
    #[must_use]
    pub fn sink(&self) -> &mut S {
        let ptr: *const S = &self.sink;
        let ptr = ptr as *mut S;
        unsafe { &mut *ptr }
    }

    #[must_use]
    pub fn into_sink(self) -> S {
        self.sink
    }
}

impl<S: Sink> ChannelLogger<'_, S> {
    #[must_use]
    pub const fn id(&self) -> usize {
        self.channel_id
    }

    #[must_use]
    pub fn sink(&self) -> &mut S {
        let ptr: *const S = self.sink;
        let ptr = ptr as *mut S;
        unsafe { &mut *ptr }
    }
}

impl<S: Sink> Logger for SimpleLogger<S> {
    fn log(&self, severity: Level, message: Arguments) {
        self.sink().consume(LogObject::new(0, severity, message))
    }
}

impl<S: Sink> Logger for ChannelLogger<'_, S> {
    fn log(&self, severity: Level, message: Arguments) {
        self.sink().consume(LogObject::new(self.channel_id, severity, message))
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::SystemTime};

    use super::*;
    use crate::{debug, log};

    #[test]
    fn test_simple() {
        let start_time = SystemTime::now();
        let logger = SimpleLogger::new(|log_object: LogObject| {
            assert!(log_object.time >= start_time);
            assert!(log_object.time <= SystemTime::now());
            assert_eq!(log_object.channel_id, 0);
            assert_eq!(log_object.severity, Level::DEBUG);
            assert_eq!(log_object.thread_id, thread::current().id());
            assert_eq!(log_object.message.as_str(), Some("message"));
        });
        debug!(logger, "message");
        log!(logger, Level::DEBUG, "message");
    }

    #[test]
    fn test_channels() {
        let mut counter = 0;
        let start_time = SystemTime::now();
        let logger = SimpleLogger::new(|log_object: LogObject| {
            assert!(log_object.time >= start_time);
            assert!(log_object.time <= SystemTime::now());
            assert_eq!(log_object.channel_id, counter);
            assert_eq!(log_object.severity, Level::DEBUG);
            assert_eq!(log_object.thread_id, thread::current().id());
            assert_eq!(log_object.message.as_str(), Some("message"));
            counter += 1;
        });
        for i in 0..10 {
            let channel = logger.channel(i);
            debug!(channel, "message");
        }
    }
}
