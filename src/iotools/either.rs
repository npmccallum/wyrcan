// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use std::io::{BufRead, Read, Write};

/// Read from or write into either of two types
#[derive(Debug)]
pub enum Either<O, T> {
    One(O),
    Two(T),
}

impl<O: Read, T: Read> Read for Either<O, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::One(x) => x.read(buf),
            Self::Two(x) => x.read(buf),
        }
    }
}

impl<O: BufRead, T: BufRead> BufRead for Either<O, T> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self {
            Self::One(x) => x.fill_buf(),
            Self::Two(x) => x.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            Self::One(x) => x.consume(amt),
            Self::Two(x) => x.consume(amt),
        }
    }
}

impl<O: Write, T: Write> Write for Either<O, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::One(x) => x.write(buf),
            Self::Two(x) => x.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::One(x) => x.flush(),
            Self::Two(x) => x.flush(),
        }
    }
}
