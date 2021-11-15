// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use super::Image;

use std::cmp::max;
use std::collections::HashMap;
use std::fmt::Display;

use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use ureq::Response;

#[derive(Clone, Debug)]
pub struct Repository {
    host: String,
    path: String,
}

impl Display for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut host = &*self.host;
        for (into, from) in Self::ALIASES {
            if &*self.host == *from && from.len() > into.len() {
                host = *into;
                break;
            }
        }

        write!(f, "{}/{}", host, self.path)
    }
}

impl Repository {
    fn auth(&self, wwwauth: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct Auth {
            token: String,
        }

        const RE: &str = "([a-z]+)=\"([^\"]+)\"";

        let mut map = HashMap::new();
        let re = Regex::new(RE).unwrap();
        for find in re.find_iter(wwwauth) {
            for caps in re.captures_iter(find.as_str()) {
                let k = caps.get(1).unwrap().as_str();
                let v = caps.get(2).unwrap().as_str();
                map.insert(k, v);
            }
        }

        let base = map.remove("realm").unwrap();
        let join: Vec<String> = map.iter().map(|(k, v)| [*k, *v].join("=")).collect();
        let args = join.join("&");
        let url = format!("{}?{}", base, args);

        let auth: Auth = ureq::get(&url).call()?.into_json()?;
        let token = format!("Bearer {}", auth.token);
        Ok(token)
    }

    pub(super) fn get(&self, path: &str, headers: &[(&str, &str)]) -> Result<Response> {
        let url = format!("https://{}/v2/{}/{}", self.host, self.path, path);

        let mut auth = false;
        let mut req = ureq::get(&url);
        for (k, v) in headers {
            req = req.set(k, v);
            auth |= *k == "Authorization";
        }

        match req.call() {
            Err(ureq::Error::Status(401, rep)) if !auth && rep.has("Www-Authenticate") => {
                let token = self.auth(rep.header("Www-Authenticate").unwrap())?;
                self.get(path, &[("Authorization", &token)])
            }

            Ok(rep) => Ok(rep),
            Err(e) => Err(e.into()),
        }
    }

    const DEFAULT_REGISTRY: &'static str = "docker.io";
    const DEFAULT_PREFIX: &'static str = "library";
    const DEFAULT_TAG: &'static str = "latest";

    const LOCALHOST: &'static str = "localhost";
    const ALIASES: &'static [(&'static str, &'static str)] =
        &[("docker.io", "registry.hub.docker.com")];

    pub fn new(mut repository: &str) -> Result<(Self, &str)> {
        // Remove any tag or digest
        let sep = repository.rfind('/').unwrap_or_default();
        let lbl = repository.rfind(':').unwrap_or_default();
        let dig = repository.rfind('@').unwrap_or_default();
        let mut tag = Self::DEFAULT_TAG;
        if lbl > sep || dig > sep {
            let (lhs, rhs) = repository.split_at(max(lbl, dig));
            repository = lhs;
            tag = &rhs[1..];
        }

        // Extract the registry
        let mut host = Self::DEFAULT_REGISTRY;
        if let Some((lhs, rhs)) = repository.find('/').map(|n| repository.split_at(n)) {
            if lhs.contains('.') || lhs.contains(':') || lhs == Self::LOCALHOST {
                repository = &rhs[1..];
                host = lhs;
            }
        }

        // Add the default prefix if necessary.
        let path = match repository.find('/') {
            None => format!("{}/{}", Self::DEFAULT_PREFIX, repository),
            _ => repository.into(),
        };

        // Substitute the aliases
        for (from, into) in Self::ALIASES {
            if host == *from {
                host = *into;
                break;
            }
        }

        let out = Self {
            host: host.into(),
            path,
        };

        Ok((out, tag))
    }

    pub fn tags(&self) -> Result<Vec<String>> {
        #[derive(Debug, Deserialize)]
        struct Tags {
            #[allow(dead_code)]
            name: String,
            tags: Vec<String>,
        }

        let rep = self.get("tags/list", &[])?;
        let tags: Tags = rep.into_json()?;
        Ok(tags.tags)
    }

    pub fn image(&self, tag: &str) -> Result<Image> {
        Image::new(self.clone(), tag)
    }
}
