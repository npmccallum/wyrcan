// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use super::Repository;
use crate::formats::{docker::v2::Layer, Manifest};

use std::fmt::Display;

use anyhow::Result;

#[derive(Clone, Debug)]
pub struct Image {
    repo: Repository,
    manifest: Manifest,
    tag: String,
}

impl Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.repo, self.tag)
    }
}

impl Image {
    pub(super) fn new(repo: Repository, tag: &str) -> Result<Self> {
        let path = format!("manifests/{}", tag);
        let rep = repo.get(&path, &[])?;

        Ok(Image {
            manifest: rep.into_json()?,
            repo,
            tag: tag.into(),
        })
    }

    pub fn layers(&self) -> Result<Vec<super::Layer>> {
        const DEFAULT: &'static str = "application/vnd.docker.image.rootfs.diff.tar.gzip";

        Ok(match &self.manifest {
            Manifest::DockerV1(m) => m
                .layers
                .iter()
                .map(|l| {
                    super::Layer::new(
                        self.repo.clone(),
                        Layer {
                            media_type: Some(DEFAULT.into()),
                            size: 0,
                            digest: l.digest.clone(),
                            urls: Vec::new(),
                        },
                    )
                })
                .collect(),

            Manifest::DockerV2(m) => m
                .layers
                .iter()
                .cloned()
                .map(|l| super::Layer::new(self.repo.clone(), l))
                .collect(),

            Manifest::DockerV2List(..) => panic!(),

            Manifest::Oci(m) => m
                .layers
                .iter()
                .map(|l| {
                    super::Layer::new(
                        self.repo.clone(),
                        Layer {
                            media_type: Some(l.media_type.clone()),
                            size: l.size,
                            digest: l.digest.clone(),
                            urls: l.urls.clone(),
                        },
                    )
                })
                .collect(),
        })
    }
}
