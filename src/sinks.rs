//! Sensible [Sink]s.

use std::time::UNIX_EPOCH;

use crate::{
    colors::{Colored, RESET_COLOR, SET_COLOR_BRIGHT_GREEN, SET_COLOR_BRIGHT_WHITE},
    filter_maps::{ChannelFilterMap, InvisibleChannelFilterMap},
    loggers::{Level, LogObject},
    writers::{StderrWriter, Write},
};

/// A trait for objects which are [LogObject] sinks.
pub trait Sink {
    /// Consumes a [LogObject] (i.e. logs it).
    fn consume(&mut self, log_object: LogObject);
}

impl<T: FnMut(LogObject)> Sink for T {
    fn consume(&mut self, log_object: LogObject) {
        self(log_object);
    }
}

/// A [Sink] that outputs formatted [LogObject]s via a [ChannelFilterMap] to a [Write].
#[derive(Clone, Copy, Debug)]
pub struct WriteSink<W: Write = StderrWriter, M: ChannelFilterMap = InvisibleChannelFilterMap> {
    /// The [ChannelFilterMap] used.
    pub channel_map: M,
    /// Whether the output should be colored.
    pub colors: bool,
    /// Whether the [ThreadId](std::thread::ThreadId) should be included in the logs.
    pub log_thread_id: bool,
    /// The sink's minimum severity level. [WriteSink] won't log [LogObject]s of lower severity.
    pub min_severity: Level,
    /// Whether the sink is muted. A muted [WriteSink] won't log anything.
    pub muted: bool,
    /// The underlying [Write].
    pub output: W,
}

impl<W: Write, M: ChannelFilterMap> WriteSink<W, M> {
    /// Constructs a new [WriteSink] with default settings (that shouldn't be relied upon).
    #[must_use]
    pub const fn new(output: W, channel_map: M) -> Self {
        Self {
            channel_map,
            colors: true,
            log_thread_id: false,
            min_severity: Level::DEBUG,
            muted: false,
            output
        }
    }
}

impl<W: Write + Default, M: ChannelFilterMap + Default> Default for WriteSink<W, M> {
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}

impl<W: Write, M: ChannelFilterMap> Sink for WriteSink<W, M> {
    fn consume(&mut self, log_object: LogObject) {
        if self.muted || log_object.severity < self.min_severity {
            return;
        }
        let Some(channel_name) = self.channel_map.filter_map(&log_object) else {
            return;
        };
        let secs_since_epoch = match log_object.time.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs() as i64,
            Err(e) => -(e.duration().as_secs() as i64),
        };
        let id: u64 = unsafe { std::mem::transmute(log_object.thread_id) };
        let _ = if self.colors {
            let level = Colored(log_object.severity).as_str();
            if self.log_thread_id {
                writeln!(self.output, "[{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{secs_since_epoch}{RESET_COLOR}][{level}][{SET_COLOR_BRIGHT_WHITE}{channel_name}{RESET_COLOR}]: {}", log_object.message)
            } else {
                writeln!(self.output, "[{SET_COLOR_BRIGHT_GREEN}{secs_since_epoch}{RESET_COLOR}][{level}{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}{channel_name}{RESET_COLOR}]: {}", log_object.message)
            }
        } else if self.log_thread_id {
            writeln!(self.output, "[{id}][{secs_since_epoch}][{}][{channel_name}]: {}", log_object.severity.as_str(), log_object.message)
        } else {
            writeln!(self.output, "[{secs_since_epoch}][{}][{channel_name}]: {}", log_object.severity.as_str(), log_object.message)
        };
    }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct MultiSink<T1: Sink, T2: Sink>(pub T1, pub T2);

impl<T1: Sink, T2: Sink> Sink for MultiSink<T1, T2> {
    fn consume(&mut self, log_object: LogObject) {
        self.0.consume(log_object);
        self.1.consume(log_object);
    }
}

/// Creates a `MultiSink` with the given given sink expressions.
#[macro_export]
macro_rules! multi_sink {
    ($head:expr, $tail:expr $(,)?) => {
        $crate::sinks::MultiSink($head, $tail)
    };

    ($head:expr, $($tail:expr),+ $(,)?) => {
        $crate::sinks::MultiSink($head, multi_sink!($($tail),+))
    };
}

#[cfg(test)]
mod tests {
    use std::{mem::MaybeUninit, time::{SystemTime, UNIX_EPOCH}, thread};

    use super::*;
    use crate::{
        colors::*,
        log, debug, info, warning, error, critical,
        loggers::{Logger, single_threaded::SimpleLogger},
    };

    fn log_to_string(f: impl FnOnce(&SimpleLogger<WriteSink<Vec<u8>>>)) -> String {
        let logger = Default::default();
        f(&logger);
        String::from_utf8(logger.into_sink().output).unwrap()
    }

    fn test_log(setup: impl FnOnce(&SimpleLogger<WriteSink<Vec<u8>>>)) -> (u64, String) {
        let mut time = MaybeUninit::uninit();
        let output = log_to_string(|logger| {
            setup(logger);
            logger.sink().output.reserve(1 << 10);
            loop {
                let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                debug!(logger, "debug");
                info!(logger, "info");
                warning!(logger, "warning");
                error!(logger, "error");
                critical!(logger, "critical");
                for i in 1..=10 {
                    log!(logger.channel(i), Level::DEBUG, "from channel {i}")
                }
                let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                if start_time == end_time {
                    time = MaybeUninit::new(start_time);
                    break;
                }
                logger.sink().output.clear();
            };
        });
        let time = unsafe { time.assume_init() };
        (time, output)
    }

    #[test]
    fn test_colorless_idless() {
        let (time, output) = test_log(|logger| {
            logger.sink().colors = false;
            logger.sink().log_thread_id = false;
        });
        let expected_output = format!(
            "[{time}][DEBUG][0]: debug\n\
             [{time}][INFO][0]: info\n\
             [{time}][WARNING][0]: warning\n\
             [{time}][ERROR][0]: error\n\
             [{time}][CRITICAL][0]: critical\n\
             [{time}][DEBUG][1]: from channel 1\n\
             [{time}][DEBUG][2]: from channel 2\n\
             [{time}][DEBUG][3]: from channel 3\n\
             [{time}][DEBUG][4]: from channel 4\n\
             [{time}][DEBUG][5]: from channel 5\n\
             [{time}][DEBUG][6]: from channel 6\n\
             [{time}][DEBUG][7]: from channel 7\n\
             [{time}][DEBUG][8]: from channel 8\n\
             [{time}][DEBUG][9]: from channel 9\n\
             [{time}][DEBUG][10]: from channel 10\n"
        );
        assert_eq!(output, expected_output);
    }

	#[test]
    fn test_colored_ided() {
        let (time, output) = test_log(|logger| {
            logger.sink().colors = true;
            logger.sink().log_thread_id = true;
        });
        let id: u64 = unsafe { std::mem::transmute(thread::current().id()) };
        let expected_output = format!(
            "[{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_CYAN}DEBUG{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}0{RESET_COLOR}]: debug\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_BLUE}INFO{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}0{RESET_COLOR}]: info\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_YELLOW}WARNING{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}0{RESET_COLOR}]: warning\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_RED}ERROR{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}0{RESET_COLOR}]: error\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_MAGENTA}CRITICAL{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}0{RESET_COLOR}]: critical\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_CYAN}DEBUG{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}1{RESET_COLOR}]: from channel 1\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_CYAN}DEBUG{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}2{RESET_COLOR}]: from channel 2\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_CYAN}DEBUG{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}3{RESET_COLOR}]: from channel 3\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_CYAN}DEBUG{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}4{RESET_COLOR}]: from channel 4\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_CYAN}DEBUG{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}5{RESET_COLOR}]: from channel 5\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_CYAN}DEBUG{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}6{RESET_COLOR}]: from channel 6\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_CYAN}DEBUG{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}7{RESET_COLOR}]: from channel 7\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_CYAN}DEBUG{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}8{RESET_COLOR}]: from channel 8\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_CYAN}DEBUG{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}9{RESET_COLOR}]: from channel 9\n\
             [{SET_COLOR_BRIGHT_WHITE}{id}{RESET_COLOR}][{SET_COLOR_BRIGHT_GREEN}{time}{RESET_COLOR}][{SET_COLOR_BRIGHT_CYAN}DEBUG{RESET_COLOR}][{SET_COLOR_BRIGHT_WHITE}10{RESET_COLOR}]: from channel 10\n"
        );
        assert_eq!(output, expected_output);
    }
}
