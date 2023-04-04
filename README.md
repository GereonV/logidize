# Logidize

A lightweight and performant logging utility

## Getting started

```rs
use logidize::{*, loggers::{Level, single_threaded::*}, sinks::WriteSink};

fn main() {
    let logger: SimpleLogger<WriteSink> = Default::default();
    debug!(logger, "{}", "Hello Debug!");
    info!(logger, "{}", "Hello Info!");
    warning!(logger, "{}", "Hello Warning!");
    error!(logger, "{}", "Hello Error!");
    critical!(logger, "{}", "Hello Critical!");
    log!(logger, Level::DEBUG, "{}", "Hello Dynamic!");
}
```

## Using default logger
```rs
use logidize::*;

fn main() {
    default_logger!().sink().unwrap()
        .channel_map.set_channel_name_or_insert_channel(0, "Main-Channel");
    debug!("logged to global logger's main-channel");
}
```

## Customizing default logger
```rs
use logidize::{*, loggers::single_threaded::*, sinks::WriteSink};

fn main() {
    let logger: SimpleLogger<WriteSink> = Default::default();
    macro_rules! default_logger { () => { logger }; }
    info!("you can change what logger the macros default to");
}
```
