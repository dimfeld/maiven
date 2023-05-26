use error_stack::{IntoReport, Report, ResultExt};
use reqwest::blocking::Client;
use serde::Deserialize;

use std::path::Path;

use super::DownloadError;

/// The simplest model config possible that gives us the model type.
#[derive(Deserialize)]
struct SimpleConfig {
    model_type: String,
}

pub fn read_model_config(dir: &Path) -> Result<String, std::io::Error> {
    let config_path = dir.join("config.json");
    let config_file = std::fs::File::open(config_path)?;
    let config: SimpleConfig = serde_json::from_reader(config_file)?;
    Ok(config.model_type)
}

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
    let data = client.get(url).send()?.json::<HuggingfaceModelInfo>()?;

    Ok(data)
}

/// Download relevant files from a Huggingface repository.
pub fn download_model(
    client: &Client,
    model_name: &str,
    destination: &Path,
) -> Result<(), Report<DownloadError>> {
    let model_info = get_model_info(client, model_name)
        .into_report()
        .attach_printable_lazy(|| format!("Fetching information for model {model_name}"))?;

    for sibling in model_info.siblings {
        if !sibling.rfilename.ends_with(".json")
            && !sibling.rfilename.ends_with(".ot")
            && !sibling.rfilename.ends_with(".txt")
        {
            continue;
        }

        let url = format!(
            "https://huggingface.co/{model_name}/resolve/main/{rfilename}",
            rfilename = sibling.rfilename
        );

        super::download_file(&client, &url, &destination.join(&sibling.rfilename))
            .into_report()
            .attach_printable(sibling.rfilename)?;
    }

    Ok(())
}
