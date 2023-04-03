pub mod colors;
pub mod filter_maps;
pub mod sinks;
pub mod writers;

use crate::multi_threaded::SimpleLogger;
use filter_maps::SimpleChannelFilterMap;
use sinks::WriteSink;
use writers::StderrWriter;

// Default::default() is not const
pub static GLOBAL_LOGGER: SimpleLogger<WriteSink<StderrWriter, SimpleChannelFilterMap<String>>> = SimpleLogger::new(
    WriteSink::new(StderrWriter, SimpleChannelFilterMap::new())
);

#[macro_export]
macro_rules! global_logger {
    () => {
        $crate::utils::GLOBAL_LOGGER
    };
}
