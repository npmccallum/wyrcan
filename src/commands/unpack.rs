// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use super::unpacker::Unpacker;
use super::Command;
use crate::api::Repository;

use std::fs::{DirBuilder, OpenOptions};
use std::io::Error;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Component, PathBuf};

use anyhow::{anyhow, Result};
use libc::{S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFMT, S_IFREG, S_IFSOCK};
use log::warn;
use structopt::StructOpt;

/// Unpacks a container into the given directory
#[derive(StructOpt, Debug)]
pub struct Unpack {
    /// The repository name (format: [source]name[:tag|@digest])
    name: String,

    /// The output directory (will be created)
    output: PathBuf,

    /// Don't display the progress bar
    #[structopt(short, long)]
    quiet: bool,
}

impl Command for Unpack {
    fn execute(self) -> Result<()> {
        std::fs::create_dir(&self.output)?;

        let (repo, tag) = Repository::new(&self.name)?;
        let image = repo.image(tag)?;
        let unpacker = Unpacker::new(&image, !self.quiet)?;

        for mut bundle in unpacker.bundles()? {
            for entry in bundle.entries()? {
                let mut entry = entry?;
                let path = entry.path()?.as_ref().to_owned();
                let head = entry.header();
                let mode = libc::mode_t::try_from(head.mode()?)?;

                // Validate path to prevent escaping chroot
                for component in path.components() {
                    match component {
                        Component::ParentDir | Component::RootDir | Component::Prefix(..) => {
                            return Err(anyhow!("disallowed component in {:?}", &path).into());
                        }

                        _ => continue,
                    }
                }

                // Append the path to our output directory.
                let into = self.output.join(&path);

                // We have a name collision. This is most likely due to a
                // case-insensitive filesystems.
                if into.exists() {
                    warn!("name collision: {:?}", into);
                    continue;
                }

                match S_IFMT & mode {
                    S_IFDIR => {
                        DirBuilder::new()
                            .mode(mode.into())
                            .recursive(false)
                            .create(into)?;
                    }

                    S_IFREG => {
                        let mut file = OpenOptions::new()
                            .create_new(true)
                            .append(true)
                            .mode(mode.into())
                            .open(into)?;

                        std::io::copy(&mut entry, &mut file)?;
                    }

                    #[cfg(target_os = "macos")]
                    S_IFCHR | S_IFBLK => {
                        warn!("skipping unsupported device: {:?}", into)
                    }

                    #[cfg(not(target_os = "macos"))]
                    S_IFCHR | S_IFBLK => {
                        let into = into.as_os_str().as_bytes().as_ptr();
                        let major = head.device_major()?.unwrap_or_default();
                        let minor = head.device_minor()?.unwrap_or_default();
                        let dev = unsafe { libc::makedev(major, minor) };
                        let ret = unsafe { libc::mknod(into as _, mode, dev) };
                        if ret < 0 {
                            return Err(Error::last_os_error().into());
                        }
                    }

                    S_IFIFO => {
                        let into = into.as_os_str().as_bytes().as_ptr();
                        let ret = unsafe { libc::mkfifo(into as _, mode) };
                        if ret < 0 {
                            return Err(Error::last_os_error().into());
                        }
                    }

                    S_IFLNK => {
                        if let Some(from) = head.link_name()? {
                            std::os::unix::fs::symlink(from, into)?;
                        } else {
                            return Err(anyhow!("link has no target: {:?}", head).into());
                        }
                    }

                    S_IFSOCK => {
                        warn!("skipping unix socket: {:?}", &path)
                    }

                    _ => return Err(anyhow!("unknown mode ({:o}) on {:?}", mode, &path).into()),
                }
            }
        }

        Ok(())
    }
}
