// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use std::cmp::min;
use std::io::{ErrorKind, Read, Result};
use std::sync::mpsc::{channel, Receiver};
use std::thread::spawn;
use std::thread::JoinHandle;

pub struct Reader {
    current: Option<(Vec<u8>, usize)>,
    thread: Option<JoinHandle<()>>,
    rx: Option<Receiver<Result<Vec<u8>>>>,
}

impl Drop for Reader {
    fn drop(&mut self) {
        self.rx.take();
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

impl Reader {
    pub fn new<R: 'static + Read + Send>(mut reader: R) -> Self {
        let (tx, rx) = channel();
        let thread = spawn(move || {
            let mut done = false;
            while !done {
                let mut buffer = vec![0; 65536];

                let result = reader.read(&mut buffer).map(|n| {
                    done = n == 0;
                    buffer.truncate(n);
                    buffer
                });

                done |= result.is_err();
                done |= tx.send(result).is_err();
            }
        });

        Self {
            current: None,
            thread: Some(thread),
            rx: Some(rx),
        }
    }

    fn pop(&mut self) -> Result<(Vec<u8>, usize)> {
        if let Some(x) = self.current.take() {
            return Ok(x);
        }

        if let Some(rx) = self.rx.as_mut() {
            if let Ok(x) = rx.recv() {
                return x.map(|b| (b, 0));
            }
        }

        Err(ErrorKind::Other.into())
    }
}

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut mark = 0;

        while mark < buf.len() {
            let (buffer, mut start) = self.pop()?;
            if buffer.len() == 0 {
                self.current = Some((buffer, start));
                break;
            }

            let input = &buffer[start..];
            let output = &mut buf[mark..];
            let len = min(input.len(), output.len());
            output[..len].copy_from_slice(&input[..len]);
            start += len;
            mark += len;

            if start < buffer.len() {
                self.current = Some((buffer, start));
            }
        }

        Ok(mark)
    }
}
