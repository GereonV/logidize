//! Sensible [Write]rs.

#[doc(no_inline)]
pub use std::io::Write;

/// A [Write] that writes to [Stderr](std::io::Stderr).
#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct StderrWriter;

impl Write for StderrWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::stderr().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stderr().flush()
    }
}

/// A [Write] that writes to [Stdout](std::io::Stdout).
#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct StdoutWriter;

impl Write for StdoutWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::stdout().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()
    }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct MultiWriter<T1: Write, T2: Write>(pub T1, pub T2);

impl<T1: Write, T2: Write> Write for MultiWriter<T1, T2> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf).and(self.1.write(buf))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush().and(self.1.flush())
    }
}

/// Creates a `MultiWriter` with the given given writer expressions.
#[macro_export]
macro_rules! multi_writer {
    ($head:expr, $tail:expr $(,)?) => {
        $crate::writers::MultiWriter($head, $tail)
    };

    ($head:expr, $($tail:expr),+ $(,)?) => {
        $crate::writers::MultiWriter($head, multi_writer!($($tail),+))
    };
}
