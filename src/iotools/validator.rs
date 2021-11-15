// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use super::Siphon;

use std::io::{Read, Write};

/// A writer that validates its input
pub trait Validatable: Write {
    /// Whether the input to the writer is valid
    fn validate(&self) -> bool;
}

/// A reader that validates the data on end-of-file
///
/// On each read, all bytes are written to the validator. When the end of the
/// file is reached, the validator will validate the data written to the
/// validator. If the data was invalid, the validator returns
/// `ErrorKind::InvalidData` instead of the end-of-file condition.
#[derive(Debug)]
pub struct Validator<R: Read, W: Validatable>(Siphon<R, W>);

impl<R: Read, W: Validatable> Validator<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self(Siphon::new(reader, writer))
    }

    pub fn reader(&self) -> &R {
        self.0.reader()
    }

    pub fn writer(&self) -> &W {
        self.0.writer()
    }
}

impl<R: Read, W: Validatable> Read for Validator<R, W> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = self.0.read(buf)?;
        if size == 0 && !self.writer().validate() {
            return Err(std::io::ErrorKind::InvalidData.into());
        }

        Ok(size)
    }
}
