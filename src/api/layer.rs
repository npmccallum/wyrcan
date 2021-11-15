// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use super::Repository;
use crate::formats::docker::v2::Layer as Level;
use crate::iotools::{Either, Validator};

use std::io::Read;

use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;

#[derive(Clone, Debug)]
pub struct Layer {
    repo: Repository,
    level: Level,
}

impl Layer {
    pub(super) fn new(repo: Repository, level: Level) -> Self {
        Self { repo, level }
    }

    pub fn decompressor<R: Read>(&self, reader: R) -> Result<Either<GzDecoder<R>, R>> {
        enum Comp {
            Gzip,
            None,
        }

        let comp = match self.level.media_type.as_deref() {
            Some("application/vnd.docker.image.rootfs.diff.tar.gzip") => Comp::Gzip,
            Some("application/vnd.docker.image.rootfs.diff.tar") => Comp::None,

            Some("application/vnd.oci.image.layer.nondistributable.v1.tar+gzip") => Comp::Gzip,
            Some("application/vnd.oci.image.layer.nondistributable.v1.tar") => Comp::None,

            Some("application/vnd.oci.image.layer.v1.tar+gzip") => Comp::Gzip,
            Some("application/vnd.oci.image.layer.v1.tar") => Comp::None,

            None => Comp::None,
            kind => return Err(anyhow!("unkown layer type: {:?}", kind)),
        };

        let x = match comp {
            Comp::Gzip => Either::One(GzDecoder::new(reader)),
            Comp::None => Either::Two(reader),
        };

        Ok(x)
    }

    pub fn download(&self) -> Result<(u64, impl Read + Send)> {
        let path = format!("blobs/{}", self.level.digest);

        let rep = self.repo.get(&path, &[])?;
        let len = rep
            .header("Content-Length")
            .and_then(|s| s.parse().ok())
            .unwrap_or(self.level.size);

        let validator = Validator::new(rep.into_reader(), self.level.digest.clone());
        Ok((len, validator))
    }
}
