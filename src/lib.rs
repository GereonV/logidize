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

    use crate::{Level, Logger};

    #[test]
    fn test_basic() {
        use super::single_threaded::{LogObject, SimpleLogger};
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
        use super::single_threaded::{LogObject, SimpleLogger};
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

    #[test]
    fn test_thread_simple() {
        use super::multi_threaded::{LogObject, SimpleLogger};
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
    fn test_thread_channel() {
        use super::multi_threaded::{LogObject, SimpleLogger};
        let mut counters = [0; 10];
        let logger = SimpleLogger::new(|log_object: LogObject| {
            let idx = match log_object.channel_id {
                None => 0,
                Some(id) => id,
            };
            counters[idx] += 1;
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
