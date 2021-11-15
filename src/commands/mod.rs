// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use structopt::StructOpt;

mod convert;
mod extract;
mod kexec;
mod tags;
mod unpack;
mod unpacker;

pub trait Command {
    fn execute(self) -> anyhow::Result<()>;
}

#[derive(StructOpt, Debug)]
#[structopt(about = "the container bootloader")]
pub enum Main {
    Tags(tags::Tags),
    Kexec(kexec::Kexec),
    Unpack(unpack::Unpack),
    Convert(convert::Convert),
}

impl Command for Main {
    fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Tags(cmd) => cmd.execute(),
            Self::Kexec(cmd) => cmd.execute(),
            Self::Unpack(cmd) => cmd.execute(),
            Self::Convert(cmd) => cmd.execute(),
        }
    }
}
