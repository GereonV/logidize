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
