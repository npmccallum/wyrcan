// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use super::extract::{Extract, LookAside};
use super::Command;
use crate::iotools::Either;

use std::fs::File;
use std::io::{sink, Sink};
use std::path::PathBuf;

use anyhow::Result;
use structopt::StructOpt;

/// Converts a container into the files necessary for boot
#[derive(StructOpt, Debug)]
pub struct Convert {
    /// The path to store the kernel
    #[structopt(short, long)]
    kernel: Option<PathBuf>,

    /// The path to store the initrd
    #[structopt(short, long)]
    initrd: Option<PathBuf>,

    /// The path to store the cmdline
    #[structopt(short, long)]
    cmdline: Option<PathBuf>,

    /// Don't display the progress bar
    #[structopt(short, long)]
    quiet: bool,

    /// The repository name (format: [source]name[:tag|@digest])
    #[structopt(help = "[source]name[:tag|@digest]")]
    name: String,
}

impl Command for Convert {
    fn execute(self) -> anyhow::Result<()> {
        fn create(value: Option<&PathBuf>) -> Result<Either<File, Sink>> {
            Ok(if let Some(path) = value {
                Either::One(File::create(path)?)
            } else {
                Either::Two(sink())
            })
        }

        let extract = Extract {
            kernel: LookAside::kernel(create(self.kernel.as_ref())?),
            initrd: create(self.initrd.as_ref())?,
            cmdline: LookAside::cmdline(create(self.cmdline.as_ref())?),
            name: self.name,
            progress: !self.quiet,
        };

        let result = extract.execute();

        if result.is_err() {
            if let Some(path) = self.kernel {
                std::fs::remove_file(path).unwrap();
            }
            if let Some(path) = self.initrd {
                std::fs::remove_file(path).unwrap();
            }
            if let Some(path) = self.cmdline {
                std::fs::remove_file(path).unwrap();
            }
        }

        result
    }
}
