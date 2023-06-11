use error_stack::{IntoReport, Report, ResultExt};
use std::path::Path;

use super::ModelError;

pub fn load_ggml_model(
    model_type: &str,
    weights_path: &Path,
) -> Result<Box<dyn llm::Model>, Report<ModelError>> {
    let model_type = match model_type {
        "bloom" => llm::ModelArchitecture::Bloom,
        "gpt2" => llm::ModelArchitecture::Gpt2,
        "gptj" => llm::ModelArchitecture::GptJ,
        "gpt-neox" => llm::ModelArchitecture::GptNeoX,
        "llama" => llm::ModelArchitecture::Llama,
        "mpt" => llm::ModelArchitecture::Mpt,
        _ => return Err(ModelError::UnknownModelType(model_type.to_string())).into_report(),
    };

    tracing::info!(
        "Loading model {} from {}",
        model_type,
        weights_path.display()
    );

    llm::load_dynamic(
        model_type,
        weights_path,
        llm::VocabularySource::Model,
        llm::ModelParameters::default(),
        |_| {},
    )
    .into_report()
    .attach_printable(weights_path.display().to_string())
    .change_context(ModelError::LoadingError)
}
