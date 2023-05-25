use serde::Deserialize;

use std::path::Path;

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
