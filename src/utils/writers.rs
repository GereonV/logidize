use std::io::Write;

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

#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct MultiWriter<T1: Write, T2: Write>(T1, T2);

impl<T1: Write, T2: Write> MultiWriter<T1, T2> {
    pub const fn new(t1: T1, t2: T2) -> Self {
        Self(t1, t2)
    }
}

impl<T1: Write, T2: Write> Write for MultiWriter<T1, T2> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf).and(self.1.write(buf))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush().and(self.1.flush())
    }
}

#[macro_export]
macro_rules! multi_writer {
    ($head:expr, $tail:expr $(,)?) => {
        MultiWriter::new($head, $tail)
    };

    ($head:expr, $($tail:expr),+ $(,)?) => {
        MultiWriter::new($head, multi_writer!($($tail),+))
    };
}
