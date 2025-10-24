use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::digest::Digest;
use crate::error::{DResult, DockerError};

#[derive(Deserialize, Debug, Clone)]
pub struct DockerTagsList {
    pub name: String,
    pub tags: BTreeSet<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DockerManifests {
    pub manifests: Vec<DockerManifest>,
    pub media_type: String,
    pub schema_version: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DockerLayers {
    pub config: DockerLayer,
    pub layers: Vec<DockerLayer>,
    pub media_type: String,
    pub schema_version: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct DockerLayer {
    #[serde(default)]
    pub data: Option<String>,
    #[serde(default)]
    pub annotations: BTreeMap<String, String>,
    pub digest: Digest,
    pub media_type: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct DockerManifest {
    #[serde(default)]
    pub annotations: BTreeMap<String, String>,
    pub digest: Digest,
    pub media_type: String,
    pub platform: DockerManifestPlatform,
    pub size: u64,
    #[serde(default)]
    pub artifact_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DockerManifestPlatform {
    pub architecture: String,
    pub os: String,
    #[serde(default, rename = "os.version")]
    pub os_version: Option<String>,
    #[serde(default, rename = "os.features")]
    pub os_features: Vec<String>,
    #[serde(default)]
    pub variant: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

impl DockerManifests {
    pub fn select(&self, os: &str, arch: &str) -> DResult<Digest> {
        for manifest in &self.manifests {
            if manifest.platform.os == os && manifest.platform.architecture == arch {
                return Ok(manifest.digest.clone());
            }
        }

        Err(DockerError::ManifestNotFound)
    }
}
