mod huggingface;

use backon::{BlockingRetryable, ExponentialBuilder};
use error_stack::{Report, ResultExt};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{
    env::VarError,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tracing::info;

use super::{LocationAndPattern, ModelParams};

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Failed writing to disk: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed writing to disk: {0}")]
    JsonWriteError(serde_json::Error),
    #[error("HTTP error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Unsupported location type")]
    UnknownLocationType,
    #[error("Invalid location {0}")]
    InvalidLocation(String),
}

impl DownloadError {
    pub fn retryable(&self) -> bool {
        match self {
            DownloadError::ReqwestError(e) => {
                !e.is_status() || e.status().map(|s| s.is_server_error()).unwrap_or(false)
            }
            _ => false,
        }
    }
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

    pub fn from_env() -> Result<Self, VarError> {
        let dir = std::env::var("MODEL_DIR")?;
        Ok(Self::new(PathBuf::from(dir)))
    }

    /// Return the directory used to store the models
    pub fn get_cache_path(&self) -> &Path {
        &self.cache_path
    }

    /// Calculate the directory that will be used for a particular model, given its source.
    pub fn get_cache_path_for_model(&self, model_remote: &str) -> PathBuf {
        let base_dir = self.get_cache_dir_for_model(model_remote);
        match model_remote.split(':').next() {
            Some("http") | Some("https") => {
                let filename = model_remote.rsplit('/').next().unwrap_or_default();
                base_dir.join(filename)
            }
            _ => base_dir,
        }
    }

    fn get_cache_dir_for_model(&self, model_remote: &str) -> PathBuf {
        let model_remote = model_remote.replace([':', '/', '\\'], "_");
        self.cache_path.join(model_remote)
    }

    fn needs_download(&self, path: &Path) -> bool {
        if !path.exists() {
            return true;
        }

        let Some(manifest) = read_manifest(path) else {
            return true;
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
    pub fn download_if_needed(
        &self,
        params: &ModelParams,
    ) -> Result<Option<PathBuf>, Report<DownloadError>> {
        let Some(dir) = params.location().map(|l| self.get_cache_dir_for_model(l)) else {
            return Ok(None);
        };

        if !self.needs_download(&dir) {
            return Ok(Some(dir));
        }

        self.download(params, &dir)?;

        Ok(Some(dir))
    }

    /// Delete any existing cached files and redownload them.
    pub fn force_download(
        &self,
        params: &ModelParams,
    ) -> Result<Option<PathBuf>, Report<DownloadError>> {
        let Some(dir) = params.location().map(|l| self.get_cache_dir_for_model(l)) else {
            return Ok(None);
        };

        self.download(params, &dir)?;

        Ok(Some(dir))
    }

    fn download_location_with_pattern(
        &self,
        loc: &LocationAndPattern,
        destination_path: &Path,
    ) -> Result<Vec<String>, Report<DownloadError>> {
        if let Some(model_name) = loc.location.strip_prefix("huggingface:") {
            huggingface::download_model(&self.client, model_name, destination_path, &loc.pattern)
        } else if loc.location.starts_with("http:") || loc.location.starts_with("https:") {
            let filename =
                loc.location.rsplit('/').next().ok_or_else(|| {
                    Report::new(DownloadError::InvalidLocation(loc.location.clone()))
                })?;
            let path = destination_path.join(filename);
            download_file(&self.client, &loc.location, &path)?;
            Ok(vec![filename.to_string()])
        } else {
            return Err(Report::new(DownloadError::UnknownLocationType))
                .attach_printable_lazy(|| loc.location.clone());
        }
    }

    fn download(
        &self,
        params: &ModelParams,
        destination_path: &Path,
    ) -> Result<(), Report<DownloadError>> {
        if destination_path.exists() {
            std::fs::remove_dir_all(destination_path).map_err(DownloadError::from)?;
        }

        let mut manifest_files = Vec::new();

        std::fs::create_dir_all(destination_path).map_err(DownloadError::from)?;

        if let Some(location) = params.location() {
            manifest_files.extend(self.download_location_with_pattern(
                &LocationAndPattern {
                    location: location.to_string(),
                    pattern: String::new(),
                },
                destination_path,
            )?);
        }

        for additional in params.additional_files() {
            manifest_files
                .extend(self.download_location_with_pattern(&additional, destination_path)?);
        }

        let manifest = Manifest {
            files: manifest_files,
        };

        let manifest_path = destination_path.join("manifest.json");
        let mut manifest_file =
            std::fs::File::create(manifest_path).map_err(DownloadError::from)?;
        serde_json::to_writer(&mut manifest_file, &manifest)
            .map_err(DownloadError::JsonWriteError)?;

        Ok(())
    }
}

/// Download a single file, creating directories if needed
fn download_file(client: &Client, url: &str, destination: &Path) -> Result<(), DownloadError> {
    let dir = destination
        .parent()
        .expect("Path has a directory and filename");
    std::fs::create_dir_all(dir)?;

    info!("Downloading {} to {}", url, destination.display());

    let dl = || -> Result<(), DownloadError> {
        let mut response = client.get(url).send()?.error_for_status()?;

        let mut file = std::fs::File::create(destination).map_err(DownloadError::from)?;
        std::io::copy(&mut response, &mut file).map_err(DownloadError::from)?;
        Ok(())
    };

    let retry = dl
        .retry(&ExponentialBuilder::default().with_min_delay(std::time::Duration::from_secs(5)))
        .when(|e| e.retryable());

    retry.call()
}

fn read_manifest(dir: &Path) -> Option<Manifest> {
    let manifest_path = dir.join("manifest.json");
    let file = std::fs::File::open(manifest_path).ok()?;
    serde_json::from_reader::<_, Manifest>(file).ok()
}

#[cfg(all(test, feature = "test-download"))]
mod test {
    use super::ModelCache;

    #[test]
    fn http() {
        let base_dir = tempfile::tempdir().expect("Creating temp dir");
        let model_cache = ModelCache::new(base_dir.path().to_path_buf());

        let model_remote =
            "https://huggingface.co/ggerganov/ggml/resolve/main/ggml-model-gpt-2-117M.bin";

        let needs_download = model_cache.needs_download(model_remote);
        assert!(needs_download, "needs_download should be initially true");

        let path = model_cache
            .download_if_needed(model_remote)
            .expect("Downloading model");

        let needs_download = model_cache.needs_download(model_remote);
        assert!(
            !needs_download,
            "needs_download should be false after download"
        );

        let full_path = path.join("ggml-model-gpt-2-117M.bin");
        assert!(full_path.exists(), "Path should exist after download");

        let manifest = super::read_manifest(&path).expect("Reading manifest");
        assert_eq!(
            manifest.files,
            vec!["ggml-model-gpt-2-117M.bin".to_string()],
            "manifest matches expected list"
        );
    }

    #[test]
    fn huggingface() {
        let base_dir = tempfile::tempdir().expect("Creating temp dir");
        let model_cache = ModelCache::new(base_dir.path().to_path_buf());
        let model_remote = "huggingface:sentence-transformers/all-MiniLM-L6-v2";

        let needs_download = model_cache.needs_download(model_remote);
        assert!(needs_download, "needs_download should be initially true");

        let path = model_cache
            .download_if_needed(model_remote)
            .expect("Downloading model");

        let needs_download = model_cache.needs_download(model_remote);
        assert!(
            !needs_download,
            "needs_download should be false after download"
        );

        let manifest = super::read_manifest(&path).expect("Reading manifest");
        // This isn't all the files, but enough to check that it's working.
        for file in [
            "1_Pooling/config.json",
            "config.json",
            "rust_model.ot",
            "tokenizer.json",
            "vocab.txt",
        ] {
            let full_path = path.join(file);
            assert!(
                full_path.exists(),
                "file {} should exist after download",
                file
            );
            assert!(
                manifest.files.contains(&file.to_string()),
                "manifest should contain {}",
                file
            );
        }

        let bin_full_path = path.join("pytorch_model.bin");
        assert!(
            !bin_full_path.exists(),
            "pickle files should not be downloaded"
        );
        assert!(
            !manifest.files.contains(&"pytorch_model.bin".to_string()),
            "manifest should not contain pytorch_model.bin"
        );
    }
}
