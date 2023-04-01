pub mod single_threaded;
pub mod multi_threaded;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    DEBUG,
    INFO,
    WARNING,
    ERROR,
    CRITICAL,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::DEBUG    => "DEBUG",
            Level::INFO     => "INFO",
            Level::WARNING  => "WARNING",
            Level::ERROR    => "ERROR",
            Level::CRITICAL => "CRITICAL",
        }
    }
}

pub trait Logger {
    fn log(&self, severity: Level, message: &str);
    fn debug(&self, message: &str) { self.log(Level::DEBUG, message); }
    fn info(&self, message: &str) { self.log(Level::INFO, message); }
    fn warning(&self, message: &str) { self.log(Level::WARNING, message); }
    fn error(&self, message: &str) { self.log(Level::ERROR, message); }
    fn critical(&self, message: &str) { self.log(Level::CRITICAL, message); }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use crate::{Level, Logger, single_threaded::{LogObject, SimpleLogger}};

    #[test]
    fn test_basic() {
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
