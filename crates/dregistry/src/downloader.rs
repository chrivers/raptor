use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};

use camino::{Utf8Path, Utf8PathBuf};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use log::info;
use reqwest::blocking::{Client, ClientBuilder};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::client::DockerClient;
use crate::digest::Digest;
use crate::error::DResult;
use crate::source::DockerSource;

pub struct DockerDownloader {
    root: Utf8PathBuf,
    client: Client,
}

impl DockerDownloader {
    const LAYER_PATH: &str = "layer";
    const MANIFEST_PATH: &str = "manifest";

    pub fn new(download_dir: Utf8PathBuf) -> DResult<Self> {
        fs::create_dir_all(download_dir.join(Self::LAYER_PATH))?;
        fs::create_dir_all(download_dir.join(Self::MANIFEST_PATH))?;

        let builder = ClientBuilder::new();
        let client = builder.build()?;

        Ok(Self {
            root: download_dir,
            client,
        })
    }

    #[must_use]
    pub fn progress_bar_style() -> ProgressStyle {
        ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn fmt::Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap();
                })
            .progress_chars("#>-")
    }

    #[must_use]
    pub fn layer_file_name(&self, digest: &Digest) -> Utf8PathBuf {
        self.root.join(Self::LAYER_PATH).join(digest.to_string())
    }

    fn manifest_file_name(&self, source: &DockerSource) -> Utf8PathBuf {
        self.root
            .join(Self::MANIFEST_PATH)
            .join(source.domain())
            .join(source.image_ref())
            .with_extension("json")
    }

    fn download_single_layer(&self, dc: &mut DockerClient, layer: &Digest) -> DResult<()> {
        let dst_file = self.layer_file_name(layer);
        let tmp_file = dst_file.with_extension("tmp");

        if fs::exists(&dst_file)? {
            /* eprintln!("Already downloaded!"); */
            return Ok(());
        }

        let mut fd = File::create(&tmp_file)?;

        let mut res = dc.blob(layer)?;

        let total_size = res.content_length().unwrap_or_default();
        let pb = ProgressBar::new(total_size);
        pb.set_style(Self::progress_bar_style());

        let mut buf = vec![0u8; 1024 * 1024];

        loop {
            let n = res.read(&mut buf)?;
            if n == 0 {
                break;
            }
            pb.inc(n as u64);
            fd.write_all(&buf[..n])?;
        }

        fs::rename(&tmp_file, &dst_file)?;

        Ok(())
    }

    fn read_json<T: DeserializeOwned>(path: &Utf8Path) -> DResult<T> {
        let fd = File::open(path)?;
        let res = serde_json::from_reader(fd)?;
        Ok(res)
    }

    fn write_json(path: &Utf8Path, data: &impl Serialize) -> DResult<()> {
        let tmp_file = path.with_extension("tmp");

        let mut fd = File::create(&tmp_file)?;
        fd.write_all(serde_json::to_string_pretty(data)?.as_bytes())?;
        fd.write_all(b"\n")?;
        drop(fd);
        fs::rename(tmp_file, path)?;

        Ok(())
    }

    pub fn pull(&self, source: &DockerSource, os: &str, arch: &str) -> DResult<Vec<Digest>> {
        info!("Logging in to registry..");
        let mut dc = DockerClient::new(self.client.clone(), source.domain(), source.image_ref())?;

        info!("Loading manifests..");
        let manifest_file = self.manifest_file_name(source);

        let manifest = if manifest_file.exists() {
            Self::read_json(&manifest_file)?
        } else {
            fs::create_dir_all(manifest_file.parent().unwrap())?;

            dc.manifest(&source.image_tag())?
        };

        let layers = dc.digests(&manifest, os, arch)?;

        info!("Downloading layers..");
        for layer in &layers {
            info!("Downloading layer {layer}");
            self.download_single_layer(&mut dc, layer)?;
        }

        Self::write_json(&manifest_file, &manifest)?;

        Ok(layers)
    }
}
