use std::borrow::Cow;

use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{ACCEPT, HeaderValue, WWW_AUTHENTICATE};
use reqwest::{IntoUrl, Method, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::api::{DockerTagsList, Manifest, V2Manifest};
use crate::authparse::parse_www_authenticate;
use crate::digest::Digest;
use crate::error::{DResult, DockerError};

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

impl DockerClient {
    const MIME_TYPE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
    const MIME_TYPE_INDEX: &str = "application/vnd.oci.image.index.v1+json";

    pub fn new(client: Client, domain: impl AsRef<str>, image: impl AsRef<str>) -> DResult<Self> {
        let image = image.as_ref().to_string();
        let domain = domain.as_ref().to_string();

        Ok(Self {
            client,
            domain,
            token: None,
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

    fn get<T: DeserializeOwned>(&mut self, url: impl IntoUrl, accept: &str) -> DResult<T> {
        /* eprintln!( */
        /*     "curl -H 'Authorization: Bearer {}' {}", */
        /*     &self.token.as_deref().unwrap_or(""), */
        /*     url.as_str() */
        /* ); */

        let url = url.into_url()?;
        let resp = self
            .request(Method::GET, url.clone())
            .header(ACCEPT, accept)
            .send()?;

        if let Some(header) = resp.headers().get(WWW_AUTHENTICATE)
            && resp.status() == StatusCode::UNAUTHORIZED
            && self.token.is_none()
        {
            self.token = Some(self.get_docker_token(header)?.token);
            return self.get(url, accept);
        }

        Ok(resp.error_for_status()?.json()?)
    }

    fn get_docker_token(&self, header: &HeaderValue) -> DResult<DockerAuthResult> {
        let mut w = parse_www_authenticate(header.to_str()?)?;

        let Some(mut settings) = w.remove("Bearer") else {
            return Err(DockerError::UnsupportedAuthMethod);
        };

        let Some(realm) = settings.remove("realm") else {
            return Err(DockerError::UnsupportedAuthMethod);
        };

        let auth_req = self.request(Method::GET, realm).query(&settings);

        Ok(auth_req.send()?.json()?)
    }

    pub fn tags(&mut self) -> DResult<DockerTagsList> {
        let url = self.api_url("tags/list");

        self.get(url, Self::MIME_TYPE_MANIFEST)
    }

    pub fn manifest(&mut self, reference: &impl Reference) -> DResult<Manifest> {
        let url = self.api_url(format!("manifests/{}", reference.reference()));

        let mime_type = [Self::MIME_TYPE_INDEX, Self::MIME_TYPE_MANIFEST].join(",");

        self.get(url, &mime_type)
    }

    pub fn blob(&mut self, digest: &Digest) -> DResult<Response> {
        let url = self.api_url(format!("blobs/{digest}"));

        Ok(self.request(Method::GET, url).send()?)
    }

    pub fn digests(&mut self, manifest: &Manifest, os: &str, arch: &str) -> DResult<Vec<Digest>> {
        let res = match manifest {
            Manifest::V1(manifest) => manifest
                .fs_layers
                .iter()
                .map(|layer| layer.blob_sum.clone())
                .collect(),

            Manifest::V2(v2 @ V2Manifest::Index { .. }) => {
                let digest = v2.select(os, arch)?;

                let manifest = self.manifest(&digest)?;
                self.digests(&manifest, os, arch)?
            }

            Manifest::V2(V2Manifest::Manifest(docker_layers)) => docker_layers
                .layers
                .iter()
                .map(|layer| layer.digest.clone())
                .collect(),
        };

        Ok(res)
    }
}
