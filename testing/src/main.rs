use const_format::concatcp;
use logidize::{*, single_threaded::*, utils::*};

fn main() {
    debug!("debug");
    info!("info");
    warning!("warning");
    error!("error");
    critical!("critical");

    let logger: SimpleLogger<WriteSink> = Default::default();
    debug!(logger, "debug");
    info!(logger, "info");
    warning!(logger, "warning");
    error!(logger, "error");
    critical!(logger, "critical");
    debug!(logger.channel(1), "from channel {}", 1);
    debug!(logger.channel(2), "from channel {}", 2);

    logger.sink().muted = true;
    debug!(logger, "muted");
    logger.sink().muted = false;
    debug!(logger, "unmuted");

    logger.sink().min_severity = Level::ERROR;
    debug!(logger, "filtered");
    info!(logger, "filtered");
    warning!(logger, "filtered");
    error!(logger, "unfiltered 1");
    critical!(logger, "unfiltered 2");

    let logger = SimpleLogger::new(WriteSink::new(StderrWriter::default(), |log_object: &LogObject| {
        const CHANNELS: [&'static str; 4] = [
            concatcp!(SET_COLOR_BRIGHT_WHITE, "Main-Channel",      RESET_COLOR),
            concatcp!(SET_COLOR_BRIGHT_WHITE, "Rendering-Channel", RESET_COLOR),
            concatcp!(SET_COLOR_BRIGHT_WHITE, "Physics-Channel",   RESET_COLOR),
            concatcp!(SET_COLOR_BRIGHT_WHITE, "Extra-Channel",     RESET_COLOR),
        ];
        let id = log_object.channel_id;
        if (log_object.severity as usize) < id {
            None
        } else {
            Some(CHANNELS[id])
        }
    }));
    logger.sink().log_thread_id = true;
    debug!(logger, "main");
    debug!(logger.channel(1), "filtered");
    info!(logger.channel(1), "rendering");
    debug!(logger.channel(2), "filtered");
    info!(logger.channel(2), "filtered");
    warning!(logger.channel(2), "physics");
    debug!(logger.channel(3), "filtered");
    info!(logger.channel(3), "filtered");
    warning!(logger.channel(3), "filtered");
    error!(logger.channel(3), "extra");

    let logger = SimpleLogger::new(WriteSink::new(multi_writer!(StderrWriter, StdoutWriter), InvisibleChannelFilterMap));
    debug!(logger, "double");
}
