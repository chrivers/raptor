use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::digest::Digest;
use crate::error::{DResult, DockerError};

#[derive(Deserialize, Debug, Clone)]
pub struct DockerTagsList {
    pub name: String,
    pub tags: BTreeSet<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Manifest {
    V1(V1Manifest),
    V2(V2Manifest),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "mediaType")]
pub enum V2Manifest {
    #[serde(rename = "application/vnd.oci.image.index.v1+json")]
    #[serde(alias = "application/vnd.docker.distribution.manifest.list.v2+json")]
    #[serde(rename_all = "camelCase")]
    Index { manifests: Vec<DockerManifest> },

    #[serde(rename = "application/vnd.oci.image.manifest.v1+json")]
    #[serde(alias = "application/vnd.docker.distribution.manifest.v2+json")]
    #[serde(rename_all = "camelCase")]
    Manifest(DockerLayers),
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
pub struct V1Manifest {
    pub name: String,
    pub tag: String,
    pub architecture: String,
    pub fs_layers: Vec<V1ManifestLayer>,

    pub history: Vec<V1ManifestHistory>,

    #[serde(default)]
    pub signatures: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct V1ManifestHistory {
    #[serde(with = "serde_nested_json")]
    pub v1_compatibility: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct V1ManifestLayer {
    pub blob_sum: Digest,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MediaType {
    #[serde(rename = "application/vnd.oci.image.layer.v1.tar+gzip")]
    #[serde(alias = "application/vnd.docker.image.rootfs.diff.tar.gzip")]
    ImageLayer,

    #[serde(rename = "application/vnd.oci.image.config.v1+json")]
    #[serde(alias = "application/vnd.docker.container.image.v1+json")]
    ImageConfig,

    #[serde(rename = "application/vnd.oci.image.manifest.v1+json")]
    #[serde(alias = "application/vnd.docker.distribution.manifest.v2+json")]
    ImageManifest,
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
