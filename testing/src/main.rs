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
}
