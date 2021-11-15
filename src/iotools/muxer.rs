// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use std::io::{Result, Write};

/// A writer which writes all input into two writers
///
/// All inputs bytes are written to the first inner writer. Then we call
/// `write_all()` on the second writer with the same buffer. Note that this
/// implies that `write()` can behave like `write_all()` as regards the second
/// inner writer. Therefore, it is desirable to put the slower or less reliable
/// writer as the first inner writer and the faster, more reliable writer as
/// the second inner writer.
#[derive(Debug)]
pub struct Muxer<U: Write, R: Write>(U, R);

impl<U: Write, R: Write> Muxer<U, R> {
    /// Create a new muxer
    ///
    /// Note that the more unreliable writer should be passed first and the
    /// more reliable writer should be passed second. For example, if you are
    /// muxing into a `TcpStream` and a `Stdout`, they should be passed in that
    /// order.
    #[inline]
    pub fn new(unreliable: U, reliable: R) -> Self {
        Self(unreliable, reliable)
    }
}

impl<U: Write, R: Write> Write for Muxer<U, R> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let size = self.0.write(buf)?;
        self.1.write_all(&buf[..size])?;
        Ok(size)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.flush()?;
        self.1.flush()
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use super::Muxer;

    #[test]
    fn muxer() {
        let mut lhs = Vec::new();
        let mut rhs = Vec::new();
        let mut mux = Muxer::new(&mut lhs, &mut rhs);

        mux.write_all(b"0123456789").unwrap();
        assert_eq!(lhs, b"0123456789");
        assert_eq!(rhs, b"0123456789");
    }
}
