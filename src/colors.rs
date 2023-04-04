//! Colored output with [Level]-wrapper.

use std::fmt::Display;

use crate::loggers::Level;
use const_format::concatcp;

/// ANSI color-code for bright red.
pub const SET_COLOR_BRIGHT_RED     : &str = "\x1b[1;31m";
/// ANSI color-code for bright green.
pub const SET_COLOR_BRIGHT_GREEN   : &str = "\x1b[1;32m";
/// ANSI color-code for bright yellow.
pub const SET_COLOR_BRIGHT_YELLOW  : &str = "\x1b[1;33m";
/// ANSI color-code for bright blue.
pub const SET_COLOR_BRIGHT_BLUE    : &str = "\x1b[1;34m";
/// ANSI color-code for bright magenta.
pub const SET_COLOR_BRIGHT_MAGENTA : &str = "\x1b[1;35m";
/// ANSI color-code for bright cyan.
pub const SET_COLOR_BRIGHT_CYAN    : &str = "\x1b[1;36m";
/// ANSI color-code for bright white.
pub const SET_COLOR_BRIGHT_WHITE   : &str = "\x1b[1;37m";
/// ANSI color-code for setting the color back to default.
pub const SET_COLOR_DEFAULT        : &str = "\x1b[39m";
/// ANSI color-code for resetting the color.
pub const RESET_COLOR              : &str = "\x1b[0m";

/// Provides colored textual representation of [Level].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Colored(pub Level);

impl Colored {
    /// Obtain the colored textual representation of the level.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self.0 {
            Level::DEBUG    => concatcp!(SET_COLOR_BRIGHT_CYAN   , Level::DEBUG.as_str()   , RESET_COLOR),
            Level::INFO     => concatcp!(SET_COLOR_BRIGHT_BLUE   , Level::INFO.as_str()    , RESET_COLOR),
            Level::WARNING  => concatcp!(SET_COLOR_BRIGHT_YELLOW , Level::WARNING.as_str() , RESET_COLOR),
            Level::ERROR    => concatcp!(SET_COLOR_BRIGHT_RED    , Level::ERROR.as_str()   , RESET_COLOR),
            Level::CRITICAL => concatcp!(SET_COLOR_BRIGHT_MAGENTA, Level::CRITICAL.as_str(), RESET_COLOR),
        }
    }
}

impl Display for Colored {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
