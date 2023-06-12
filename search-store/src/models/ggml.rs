use error_stack::{IntoReport, Report, ResultExt};
use std::path::{Path, PathBuf};

use super::ModelError;

pub fn load_ggml_model(
    model_name: &str,
    model_type: &str,
    weights_path: &Path,
    vocab_path: Option<PathBuf>,
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
        model_name,
        weights_path.display()
    );

    let vocab_source = vocab_path
        .map(llm::VocabularySource::HuggingFaceTokenizerFile)
        .unwrap_or(llm::VocabularySource::Model);

    llm::load_dynamic(
        model_type,
        weights_path,
        vocab_source,
        llm::ModelParameters::default(),
        |_| {},
    )
    .into_report()
    .attach_printable(weights_path.display().to_string())
    .change_context(ModelError::LoadingError)
}
