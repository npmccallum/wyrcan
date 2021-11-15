// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

mod digest;
pub mod docker;
pub mod oci;

pub use self::digest::Digest;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Manifest {
    #[serde(rename = "application/vnd.docker.distribution.manifest.v1+json")]
    #[serde(alias = "application/vnd.docker.distribution.manifest.v1+prettyjws")]
    DockerV1(docker::v1::Manifest),

    #[serde(rename = "application/vnd.docker.distribution.manifest.v2+json")]
    DockerV2(docker::v2::Manifest),

    #[serde(rename = "application/vnd.docker.distribution.manifest.list.v2+json")]
    DockerV2List(docker::v2::ManifestList),

    #[serde(rename = "application/vnd.oci.image.manifest.v1+json")]
    Oci(oci::Manifest),
}
