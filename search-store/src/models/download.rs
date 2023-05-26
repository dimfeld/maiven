mod huggingface;

use error_stack::{Report, ResultExt};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Failed writing to disk: {0}")]
    IoError(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Unsupported location type")]
    UnknownLocationType,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    files: Vec<String>,
}

pub struct ModelCache {
    cache_path: PathBuf,
    client: Client,
}

impl ModelCache {
    pub fn new(cache_path: PathBuf) -> Self {
        ModelCache {
            cache_path,
            client: Client::new(),
        }
    }

    pub fn get_cache_path(&self) -> &Path {
        &self.cache_path
    }

    pub fn get_cache_path_for_model(&self, model_remote: &str) -> PathBuf {
        let model_remote = model_remote.replace([':', '/', '\\'], "_");
        self.cache_path.join(model_remote)
    }

    pub fn needs_download(&self, model: &str) -> bool {
        let path = self.get_cache_path_for_model(model);
        if !path.exists() {
            return true;
        }

        let manifest_path = path.join("manifest.json");
        let Ok(file) =  std::fs::File::open(manifest_path) else {
            return false;
        };

        let Ok(manifest) = serde_json::from_reader::<_, Manifest>(file) else {
            return false;
        };

        for file in manifest.files {
            let file_path = path.join(file);
            if !file_path.exists() {
                return true;
            }
        }

        false
    }

    /// Check if the files for this model have been downloaded, and download them if needed.
    pub fn download_if_needed(&self, model: &str) -> Result<PathBuf, Report<DownloadError>> {
        let path = self.get_cache_path_for_model(model);
        if path.exists() {
            return Ok(path);
        }

        self.download(model, &path)?;

        Ok(path)
    }

    /// Delete any existing cached files and redownload them.
    pub fn force_download(&self, model: &str) -> Result<PathBuf, Report<DownloadError>> {
        let path = self.get_cache_path_for_model(model);

        self.download(model, &path)?;

        Ok(path)
    }

    fn download(
        &self,
        model_remote: &str,
        destination_path: &Path,
    ) -> Result<(), Report<DownloadError>> {
        if destination_path.exists() {
            std::fs::remove_dir_all(destination_path).map_err(DownloadError::from)?;
        }

        std::fs::create_dir_all(destination_path).map_err(DownloadError::from)?;
        if let Some(model_name) = model_remote.strip_prefix("huggingface:") {
            huggingface::download_model(&self.client, model_name, destination_path)?;
        } else if model_remote.starts_with("http:") || model_remote.starts_with("https:") {
            let filename = model_remote.split('/').last().unwrap();
            let path = destination_path.join(filename);
            download_file(&self.client, model_remote, &path)?;
        } else {
            return Err(Report::new(DownloadError::UnknownLocationType))
                .attach_printable_lazy(|| model_remote.to_string());
        }

        Ok(())
    }
}

/// Download a single file, creating directories if needed
fn download_file(client: &Client, url: &str, destination: &Path) -> Result<(), DownloadError> {
    let dir = destination
        .parent()
        .expect("Path has a directory and filename");
    std::fs::create_dir_all(dir)?;

    let mut response = client.get(url).send()?;
    let mut file = std::fs::File::create(destination)?;
    std::io::copy(&mut response, &mut file)?;
    Ok(())
}
