// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use std::collections::HashMap;

use serde::Deserialize;

use super::Digest;

#[derive(Clone, Debug, Deserialize)]
pub struct Descriptor {
    #[serde(rename = "mediaType")]
    pub media_type: String,

    pub digest: Digest,

    pub size: u64,

    #[serde(default)]
    pub urls: Vec<String>,

    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: usize,

    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,

    pub config: Descriptor,

    pub layers: Vec<Descriptor>,

    #[serde(default)]
    pub annotations: HashMap<String, String>,
}
