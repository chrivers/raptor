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

#[cfg(test)]
mod tests {
    use crate::{digest::Digest, reference, source::DockerSource};

    #[test]
    fn parse_combinations() {
        let digest = Digest::parse(&format!("sha256:{}", "0".repeat(64))).unwrap();

        for host in [None, Some("example.org".into())] {
            for port in [None, Some(8080)] {
                for namespace in [None, Some("namespace".into())] {
                    for tag in [None, Some("tag".into())] {
                        for digest in [None, Some(digest.clone())] {
                            // can't have a port number without a host
                            if port.is_some() && host.is_none() {
                                continue;
                            }

                            // can't have a tag and a digest at the same time
                            if tag.is_some() && digest.is_some() {
                                continue;
                            }

                            let src = DockerSource {
                                host: host.clone(),
                                port,
                                namespace: namespace.clone(),
                                repository: "debian".into(),
                                tag: tag.clone(),
                                digest: digest.clone(),
                            };

                            let name = src.to_string();

                            let dst = reference::parse(&name).unwrap();

                            println!("{src:?}");
                            assert_eq!(src, dst);
                        }
                    }
                }
            }
        }
    }
}
