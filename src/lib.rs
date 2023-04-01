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
