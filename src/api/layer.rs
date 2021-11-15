use super::Repository;
use crate::formats::docker::v2::Layer as Level;
use crate::formats::Digest;
use crate::iotools::{Either, Validator};

use std::{collections::HashMap, io::Read};

use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use reqwest::blocking::Response;

#[derive(Clone, Debug)]
pub struct Layer {
    repo: Repository,
    level: Level,
}

impl Layer {
    pub(super) fn new(repo: Repository, level: Level) -> Self {
        Self { repo, level }
    }

    pub fn size(&self) -> u64 {
        self.level.size
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

    pub fn download(&self) -> Result<Validator<Response, Digest>> {
        let path = format!("blobs/{}", self.level.digest);

        let rep = self.repo.get(&path, HashMap::new())?;
        if self.level.size != 0 && rep.content_length() != Some(self.level.size) {
            return Err(anyhow!("unexpected size: {:?}", rep.content_length()));
        }

        Ok(Validator::new(rep, self.level.digest.clone()))
    }
}
