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

// #[cfg(test)]
// mod tests {
// 	use crate::single_threaded::SimpleLogger;
// 	use super::*;
// 
// 	#[test]
// 	fn test_basics() {
// 		let mut output = Vec::<u8>::new();
// 		let logger = SimpleLogger::new(WriteSink::new(&mut output, InvisibleChannelFilterMap));
// 		debug!(logger, "debug");
// 		info!(logger, "info");
// 		warning!(logger, "warning");
// 		error!(logger, "error");
// 		critical!(logger, "critical");
// 		debug!(logger.channel(1), "from channel {}", 1);
// 		debug!(logger.channel(2), "from channel {}", 2);
// 		let mut iter = output.iter();
// 		let _ = iter.by_ref().filter(|char| **char == b'm').take(3).last();
// 		assert_eq!(std::str::from_utf8(&iter.as_slice()[..5]).unwrap(), "DEBUG");
// 		let _ = iter.by_ref().filter(|char| **char == b'm').take(2).last();
// 		assert_eq!(std::str::from_utf8(&iter.as_slice()[..1]).unwrap(), "0");
// 		let _ = iter.by_ref().filter(|char| **char == b'm').take(1).last();
// 		assert_eq!(std::str::from_utf8(&iter.as_slice()[..9]).unwrap(), "]: debug\n");
// 
// 		let _ = iter.by_ref().filter(|char| **char == b'm').take(3).last();
// 		assert_eq!(std::str::from_utf8(&iter.as_slice()[..4]).unwrap(), "INFO");
// 		let _ = iter.by_ref().filter(|char| **char == b'm').take(2).last();
// 		assert_eq!(std::str::from_utf8(&iter.as_slice()[..1]).unwrap(), "0");
// 		let _ = iter.by_ref().filter(|char| **char == b'm').take(1).last();
// 		assert_eq!(std::str::from_utf8(&iter.as_slice()[..8]).unwrap(), "]: info\n");
// 
// 		let _ = iter.by_ref().filter(|char| **char == b'\n').take(4).last();
// 		let _ = iter.by_ref().filter(|char| **char == b'm').take(5).last();
// 		assert_eq!(std::str::from_utf8(&iter.as_slice()[..1]).unwrap(), "1");
// 		let _ = iter.by_ref().filter(|char| **char == b'm').take(1).last();
// 		assert_eq!(std::str::from_utf8(&iter.as_slice()[..18]).unwrap(), "]: from channel 1\n");
// 	}
// }
