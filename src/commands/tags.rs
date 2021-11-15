// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use crate::api::Repository;

use super::Command;

use structopt::StructOpt;

/// List all tags for a given repository
#[derive(StructOpt, Debug)]
pub struct Tags {
    /// The repository name (format: [source]name)
    name: String,
}

impl Command for Tags {
    fn execute(self) -> anyhow::Result<()> {
        let (repo, ..) = Repository::new(&self.name)?;

        for tag in repo.tags()? {
            println!("{}", tag);
        }

        Ok(())
    }
}
