use std::{io::Write, time::UNIX_EPOCH};

use crate::{Level, LogObject, Sink};
use super::{colors::{Colored, RESET_COLOR, SET_COLOR_BRIGHT_GREEN, SET_COLOR_BRIGHT_WHITE}, filter_maps::{ChannelFilterMap, InvisibleChannelFilterMap}, writers::StderrWriter};

#[derive(Clone, Copy, Debug)]
pub struct WriteSink<W: Write = StderrWriter, M: ChannelFilterMap = InvisibleChannelFilterMap> {
    pub channel_map: M,
    pub colors: bool,
    pub log_thread_id: bool,
    pub min_severity: Level,
    pub muted: bool,
    pub output: W,
}

impl<W: Write, M: ChannelFilterMap> WriteSink<W, M> {
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

#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct MultiSink<T1: Sink, T2: Sink>(pub T1, pub T2);

impl<T1: Sink, T2: Sink> Sink for MultiSink<T1, T2> {
    fn consume(&mut self, log_object: LogObject) {
        self.0.consume(log_object);
        self.1.consume(log_object);
    }
}

#[macro_export]
macro_rules! multi_sink {
    ($head:expr, $tail:expr $(,)?) => {
        MultiSink($head, $tail)
    };

    ($head:expr, $($tail:expr),+ $(,)?) => {
        MultiSink($head, multi_sink!($($tail),+))
    };
}

#[cfg(test)]
mod tests {
    use std::{fmt::Write, mem::MaybeUninit, time::{SystemTime, UNIX_EPOCH}};

    use crate::{single_threaded::SimpleLogger, Level, Logger, log, debug, info, warning, error, critical};

    use super::WriteSink;

    fn log_to_string(f: impl FnOnce(&SimpleLogger<WriteSink<Vec<u8>>>)) -> String {
        let logger = Default::default();
        f(&logger);
        String::from_utf8(logger.into_sink().output).unwrap()
    }

    #[test]
    fn test_colorless_idless() {
        let mut time = MaybeUninit::uninit();
        let output = log_to_string(|logger| {
            logger.sink().output.reserve(1 << 10);
            logger.sink().colors = false;
            logger.sink().log_thread_id = false;
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
            }
        });
        let time = unsafe { time.assume_init() };
        let mut expected_output = String::new();
        let _ = write!(&mut expected_output,
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
}
