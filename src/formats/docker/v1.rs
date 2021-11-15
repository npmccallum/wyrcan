// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use serde::Deserialize;

use crate::formats::Digest;

#[derive(Clone, Debug, Deserialize)]
pub struct History {}

#[derive(Clone, Debug, Deserialize)]
pub struct Layer {
    #[serde(rename = "blobSum")]
    pub digest: Digest,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: usize,

    pub name: String,

    pub tag: String,

    pub architecture: String,

    #[serde(rename = "fsLayers")]
    pub layers: Vec<Layer>,

    #[serde(default)]
    pub history: Vec<History>,
}
