// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use super::super::Digest;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Platform {
    pub architecture: String,

    pub os: String,

    #[serde(rename = "os.version")]
    pub os_version: Option<String>,

    #[serde(default, rename = "os.features")]
    pub os_features: Vec<String>,

    pub variant: Option<String>,

    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Item {
    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,

    pub size: u64,

    pub digest: Digest,

    pub platform: Platform,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ManifestList {
    #[serde(rename = "schemaVersion")]
    pub schema_version: usize,

    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,

    pub manifests: Vec<Item>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,

    pub size: u64,

    pub digest: Digest,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Layer {
    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,

    pub size: u64,

    pub digest: Digest,

    #[serde(default)]
    pub urls: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: usize,

    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,

    pub config: Config,

    #[serde(default)]
    pub layers: Vec<Layer>,
}
