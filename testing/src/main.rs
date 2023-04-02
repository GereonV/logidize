use logidize::{Logger, single_threaded::*, utils::*};

fn main() {
    let logger = SimpleLogger::new(StdErrSink::new(|id| Some(id)));
    logger.debug("debug");
    logger.info("info");
    logger.warning("warning");
    logger.error("error");
    logger.critical("critical");

    let channel = logger.channel(1);
    channel.debug("from channel 1");

    logger.sink().mute();
    logger.debug("muted");
    logger.sink().unmute();
    logger.debug("unmuted");
}
