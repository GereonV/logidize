use logidize::{
    *,
    loggers::{Level, Logger, single_threaded::*},
    filter_maps::{InvisibleChannelFilterMap, SimpleChannel, StaticSeverityChannelFilterMap},
    sinks::WriteSink,
    writers::{StderrWriter, StdoutWriter}
};

fn main() {
    debug!("disabled channel");
    info!("disabled channel");
    warning!("disabled channel");
    error!("disabled channel");
    critical!("disabled channel");
    default_logger!().sink().unwrap().channel_map.insert_channel(0, SimpleChannel::new("Main-Channel".into()));
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

    let logger = SimpleLogger::new(WriteSink::new(
        StderrWriter,
        StaticSeverityChannelFilterMap(&[
            ("Main-Channel"     , Level::INFO),
            ("Rendering-Channel", Level::WARNING),
            ("Physics-Channel"  , Level::ERROR),
            ("Extra-Channel"    , Level::CRITICAL),
        ])
    ));
    logger.sink().log_thread_id = true;
    debug!(logger, "filtered");
    info!(logger, "main");
    debug!(logger.channel(1), "filtered");
    info!(logger.channel(1), "filtered");
    warning!(logger.channel(1), "rendering");
    debug!(logger.channel(2), "filtered");
    info!(logger.channel(2), "filtered");
    warning!(logger.channel(2), "filtered");
    error!(logger.channel(2), "physics");
    debug!(logger.channel(3), "filtered");
    info!(logger.channel(3), "filtered");
    warning!(logger.channel(3), "filtered");
    error!(logger.channel(3), "filtered");
    critical!(logger.channel(3), "extra");

    let logger = SimpleLogger::new(WriteSink::new(multi_writer!(StderrWriter, StdoutWriter), InvisibleChannelFilterMap));
    debug!(logger, "double");
}
