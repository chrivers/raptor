use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;

use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header;
use reqwest::{IntoUrl, Method};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::api::{DockerLayers, DockerManifests, DockerTagsList};
use crate::digest::Digest;
use crate::error::DResult;

pub trait Reference {
    fn reference(&self) -> Cow<str>;
}

impl Reference for &str {
    fn reference(&self) -> Cow<str> {
        Cow::Borrowed(self)
    }
}

impl Reference for String {
    fn reference(&self) -> Cow<str> {
        Cow::Borrowed(self)
    }
}

impl Reference for Digest {
    fn reference(&self) -> Cow<str> {
        Cow::Owned(self.to_string())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct DockerAuthResult {
    token: String,
}

pub struct DockerClient {
    client: Client,
    domain: String,
    token: Option<String>,
    image: String,
}

pub struct TokenProvider<'a> {
    pub domain: &'a str,
    pub service: &'a str,
}

pub static TOKEN_PROVIDERS: LazyLock<HashMap<&'static str, TokenProvider<'static>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert(
            "index.docker.io",
            TokenProvider {
                domain: "auth.docker.io",
                service: "registry.docker.io",
            },
        );

        map.insert(
            "ghcr.io",
            TokenProvider {
                domain: "ghcr.io",
                service: "ghcr.io",
            },
        );

        map
    });

impl DockerClient {
    const MIME_TYPE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
    const MIME_TYPE_INDEX: &str = "application/vnd.oci.image.index.v1+json";

    pub fn new(client: Client, domain: impl AsRef<str>, image: impl AsRef<str>) -> DResult<Self> {
        let image = image.as_ref().to_string();
        let domain = domain.as_ref().to_string();

        let token = Self::get_docker_token(&client, &domain, &image)?;

        Ok(Self {
            client,
            domain,
            token,
            image,
        })
    }

    fn request(&self, method: Method, url: impl IntoUrl) -> RequestBuilder {
        let mut res = self.client.request(method, url);

        if let Some(token) = &self.token {
            res = res.header("Authorization", format!("Bearer {token}"));
        }

        res
    }

    fn api_url(&self, url: impl AsRef<str>) -> String {
        let domain = &self.domain;
        let image = &self.image;
        let url = url.as_ref();

        format!("https://{domain}/v2/{image}/{url}")
    }

    fn get<T: DeserializeOwned>(&self, url: impl IntoUrl, accept: &str) -> DResult<T> {
        /* eprintln!( */
        /*     "curl -H 'Authorization: Bearer {}' {}", */
        /*     &self.token.as_deref().unwrap_or(""), */
        /*     url.as_str() */
        /* ); */

        let url = url.into_url()?;
        let res = self
            .request(Method::GET, url)
            .header(header::ACCEPT, accept)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(res)
    }

    fn get_docker_token(client: &Client, domain: &str, image: &str) -> DResult<Option<String>> {
        let Some(TokenProvider { domain, service }) = TOKEN_PROVIDERS.get(domain) else {
            return Ok(None);
        };

        let res = client
            .get(format!(
                "https://{domain}/token?service={service}&scope=repository:{image}:pull",
            ))
            .send()?;

        let js = res.error_for_status()?.json::<DockerAuthResult>()?;

        Ok(Some(js.token))
    }

    pub fn tags(&self) -> DResult<DockerTagsList> {
        let url = self.api_url("tags/list");

        self.get(url, Self::MIME_TYPE_MANIFEST)
    }

    pub fn manifests(&self, reference: &str) -> DResult<DockerManifests> {
        let url = self.api_url(format!("manifests/{reference}"));

        self.get(url, Self::MIME_TYPE_INDEX)
    }

    pub fn layers(&self, hash: &Digest) -> DResult<DockerLayers> {
        let url = self.api_url(format!("manifests/{hash}",));

        self.get(url, Self::MIME_TYPE_MANIFEST)
    }

    pub fn blob(&self, digest: &Digest) -> DResult<Response> {
        let url = self.api_url(format!("blobs/{digest}"));

        Ok(self.request(Method::GET, url).send()?)
    }
}
