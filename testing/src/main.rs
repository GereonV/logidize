use logidize::{*, single_threaded::*, utils::*};

fn main() {
    let logger: SimpleLogger<StdErrSink> = Default::default();
    logger.debug("debug");
    logger.info("info");
    logger.warning("warning");
    logger.error("error");
    logger.critical("critical");
    logger.channel(1).debug("from channel 1");
    logger.channel(2).debug("from channel 2");

    logger.sink().muted = true;
    logger.debug("muted");
    logger.sink().muted = false;
    logger.debug("unmuted");

    logger.sink().min_severity = Level::ERROR;
    logger.debug("filtered");
    logger.info("filtered");
    logger.warning("filtered");
    logger.error("unfiltered 1");
    logger.critical("unfiltered 2");

    let logger = SimpleLogger::new(StdErrSink::new(|log_object: &LogObject| {
        const CHANNELS: [&'static str; 4] = [
            "Main-Channel",
            "Rendering-Channel",
            "Physics-Channel",
            "Extra-Channel",
        ];
        let id = log_object.channel_id;
        if (log_object.severity as usize) < id {
            None
        } else {
            Some(CHANNELS[id])
        }
    }));
    logger.debug("main");
    logger.channel(1).debug("filtered");
    logger.channel(1).info("rendering");
    logger.channel(2).debug("filtered");
    logger.channel(2).info("filtered");
    logger.channel(2).warning("physics");
    logger.channel(3).debug("filtered");
    logger.channel(3).info("filtered");
    logger.channel(3).warning("filtered");
    logger.channel(3).error("extra");
}
