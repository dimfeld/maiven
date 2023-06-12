use error_stack::{IntoReport, Report, ResultExt};
use reqwest::blocking::Client;
use serde::Deserialize;

use std::path::Path;

use super::DownloadError;

#[derive(Deserialize, Debug)]
pub struct HuggingFaceSibling {
    rfilename: String,
}

/// Partial info for a Huggingface model.
#[derive(Deserialize, Debug)]
pub struct HuggingfaceModelInfo {
    #[serde(rename = "modelId")]
    pub model_id: String,
    /// Siblings are actually the files in the repository
    pub siblings: Vec<HuggingFaceSibling>,
    #[serde(rename = "lastModified")]
    pub last_modified: String,
}

/// Retrieve model information from the Huggingface API.
pub fn get_model_info(
    client: &Client,
    model_name: &str,
) -> Result<HuggingfaceModelInfo, DownloadError> {
    let url = format!("https://huggingface.co/api/models/{model_name}");
    let data = client
        .get(url)
        .send()?
        .error_for_status()?
        .json::<HuggingfaceModelInfo>()?;

    Ok(data)
}

/// Download relevant files from a Huggingface repository.
pub fn download_model(
    client: &Client,
    model_name: &str,
    destination: &Path,
    pattern: &str,
) -> Result<Vec<String>, Report<DownloadError>> {
    let model_info = get_model_info(client, model_name)
        .into_report()
        .attach_printable_lazy(|| format!("Fetching information for model {model_name}"))?;

    let match_pattern = if pattern.is_empty() {
        r##"\.(json|md|ot|txt)$"##
    } else {
        pattern
    };

    let matcher = regex::Regex::new(match_pattern)
        .into_report()
        .change_context_lazy(|| {
            DownloadError::InvalidLocation(format!("Invalid regex pattern: {}", match_pattern))
        })?;

    let mut files = Vec::new();

    for sibling in model_info.siblings {
        if !matcher.is_match(&sibling.rfilename) {
            continue;
        }

        let url = format!(
            "https://huggingface.co/{model_name}/resolve/main/{rfilename}",
            rfilename = sibling.rfilename
        );

        super::download_file(client, &url, &destination.join(&sibling.rfilename))
            .into_report()
            .attach_printable_lazy(|| sibling.rfilename.clone())?;

        files.push(sibling.rfilename);
    }

    Ok(files)
}
