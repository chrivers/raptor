use std::fmt::Display;

use crate::digest::Digest;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct DockerSource {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub namespace: Option<String>,
    pub repository: String,
    pub tag: Option<String>,
    pub digest: Option<Digest>,
}

impl DockerSource {
    #[must_use]
    pub fn image_ref(&self) -> String {
        format!(
            "{}/{}",
            self.namespace.as_deref().unwrap_or("library"),
            &self.repository
        )
    }

    #[must_use]
    pub fn image_tag(&self) -> &str {
        self.tag.as_deref().unwrap_or("latest")
    }
}

impl Display for DockerSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(host) = &self.host {
            write!(f, "{host}")?;
        }

        if let Some(port) = &self.port {
            write!(f, ":{port}")?;
        }

        if self.host.is_some() {
            write!(f, "/")?;
        }

        if let Some(namespace) = &self.namespace {
            write!(f, "{namespace}/")?;
        }

        write!(f, "{}", self.repository)?;

        if let Some(tag) = &self.tag {
            write!(f, ":{tag}")?;
        }

        if let Some(digest) = &self.digest {
            write!(f, "@{digest}")?;
        }

        Ok(())
    }
}
