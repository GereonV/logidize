// loggers (configuration at compile-time)
//   - channels
//   - levels
//   - output/formatting function
// - filtering
//   - by channel globally
//   - by level per channel or globally
// - custom output function (that formats) supplied with:
//   - time
//   - thread
//   - level
//   - channel

pub use macros::levels;

pub struct Levels<const N: usize> {
    pub data: [(&'static str, usize); N],
}

const DEFAULT_LEVELS: Levels<5> = levels![
    0 => "DEBUG",
    1 => "INFO",
    2 => "WARNING",
    3 => "ERROR",
    4 => "CRITICAL",
];
