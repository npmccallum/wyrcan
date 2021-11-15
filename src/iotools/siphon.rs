// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use std::io::{Read, Result, Write};

/// A reader which writes to a writer on every read
///
/// On every read, calls `write_all()` on the writer. This should generally be
/// used with reliable writers.
#[derive(Debug)]
pub struct Siphon<R: Read, W: Write>(R, W);

impl<R: Read, W: Write> Siphon<R, W> {
    /// Creates a new siphoner
    #[inline]
    pub fn new(reader: R, writer: W) -> Self {
        Self(reader, writer)
    }

    pub fn reader(&self) -> &R {
        &self.0
    }

    pub fn writer(&self) -> &W {
        &self.1
    }
}

impl<R: Read, W: Write> Read for Siphon<R, W> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let size = self.0.read(buf)?;
        self.1.write_all(&buf[..size])?;
        Ok(size)
    }
}

#[cfg(test)]
mod test {
    use std::io::Read;

    use super::Siphon;

    #[test]
    fn muxer() {
        let arr = b"0123456789";
        let mut dst = Vec::new();
        let mut all = Vec::new();
        let mut src = &arr[..];
        let mut sip = Siphon::new(&mut src, &mut dst);

        let len = sip.read_to_end(&mut all).unwrap();
        assert_eq!(len, arr.len());
        assert_eq!(all, arr);
    }
}
