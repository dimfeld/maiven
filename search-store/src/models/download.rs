use std::path::{Path, PathBuf};

pub struct ModelDownloader {
    pub name: String,
    cache_path: PathBuf,
    remote_path: String,
}

impl ModelDownloader {
    pub fn new(name: String, cache_path: PathBuf, remote_path: String) -> Self {
        ModelDownloader {
            name,
            cache_path,
            remote_path,
        }
    }

    pub async fn needs_download(&self) -> bool {
        todo!()
    }

    pub async fn download_if_needed(&self) -> Result<(), ()> {
        todo!()
    }

    pub async fn force_download(&self) -> Result<(), ()> {
        todo!()
    }

    async fn download(&self) -> Result<(), ()> {
        todo!()
    }
}

pub fn download_huggingface(destination: &Path, repo: &str) {
    todo!()
}

pub fn download_http(destination: &Path, url: &str) {
    todo!()
}
