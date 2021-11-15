use super::unpacker::Unpacker;
use crate::api::Repository;
use crate::iotools::Muxer;
use super::Command;

use std::{path::PathBuf};
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{anyhow, Result};
use libc::{S_IFLNK, S_IFMT, S_IFREG};
use tar::{Header};

#[derive(Debug)]
pub struct LookAside<O: Write> {
    symlink: PathBuf,
    output: O,
}

impl<O: Write> LookAside<O> {
    const PREFIX: &'static str = "boot";

    pub fn kernel(output: O) -> Self {
        Self {
            symlink: Path::new(Self::PREFIX).join("wyrcan.kernel"),
            output,
        }
    }

    pub fn cmdline(output: O) -> Self {
        Self {
            symlink: Path::new(Self::PREFIX).join("wyrcan.cmdline"),
            output,
        }
    }

    fn glance(&mut self, header: &Header) -> Result<Option<&mut dyn Write>> {
        let mode: libc::mode_t = header.mode()?.try_into()?;
        let path = header.path()?;

        if path.as_ref() == &self.symlink {
            match S_IFMT & mode {
                S_IFREG => return Ok(Some(&mut self.output)),

                S_IFLNK => {
                    if let Some(link) = header.link_name()? {
                        if link.is_relative() && link.components().count() == 1 {
                            self.symlink = Path::new(Self::PREFIX).join(link.as_ref());
                            return Ok(None);
                        }
                    }
                }

                _ => ()
            }

            Err(anyhow!("unsupported entry: {:?}", header))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub struct Extract<K: Write, I: Write, C: Write> {
    pub kernel: LookAside<K>,
    pub initrd: I,
    pub cmdline: LookAside<C>,
    pub name: String,
    pub progress: bool,
}

impl<K: Write, I: Write, C: Write> Command for Extract<K, I, C> {
    fn execute(self) -> anyhow::Result<()> {
        let (repo, tag) = Repository::new(&self.name)?;
        let image = repo.image(tag)?;
        let unpacker = Unpacker::new(&image, self.progress)?;

        let mut kernel = self.kernel;
        let mut initrd = self.initrd;
        let mut cmdline = self.cmdline;
        for mut bundle in unpacker.bundles()? {
            for entry in bundle.entries()? {
                let mut entry = entry?;
                let head = entry.header().clone();
                let path = entry.path()?;
                let mode: libc::mode_t = head.mode()?.try_into()?;
                let size = head.size()?.try_into()?;

                // Create an entry in the cpio.
                let mut builder = cpio::newc::Builder::new(path.to_str().unwrap());
                builder = builder.mode(head.mode()?);
                builder = builder.uid(head.uid()?.try_into()?);
                builder = builder.gid(head.gid()?.try_into()?);
                builder = builder.mtime(head.mtime()?.try_into()?);
                if let Some(maj) = head.device_major()? {
                    builder = builder.dev_major(maj);
                }
                if let Some(min) = head.device_minor()? {
                    builder = builder.dev_minor(min);
                }

                // Handle symlinks.
                let link = head.link_name_bytes();
                let mut link = link.as_deref().unwrap_or(&[]);
                let (mut reader, size): (&mut dyn Read, _) = if mode & S_IFMT == S_IFLNK {
                    let len = link.len().try_into()?;
                    (&mut link, len)
                } else {
                    (&mut entry, size)
                };

                // Create the output writer.
                let writer = builder.write(&mut initrd, size);

                // Possibly copy data to one of our lookasides.
                let mut sink = std::io::sink();
                let mut muxer: Muxer<_, &mut dyn Write> = match kernel.glance(&head)? {
                    Some(w) => Muxer::new(writer, w),
                    None => match cmdline.glance(&head)? {
                        Some(w) => Muxer::new(writer, w),
                        None => Muxer::new(writer, &mut sink),
                    }
                };

                // Copy from the tarball to the cpio.
                std::io::copy(&mut reader, &mut muxer)?;
            }
        }

        Ok(())
    }
}
