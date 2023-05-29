use std::path::Path;

pub fn load_ggml_model(model_type: &str, weights_path: &Path) -> Box<dyn llm::Model> {
    let model_type = match model_type {
        "bloom" => llm::ModelArchitecture::Bloom,
        "gpt2" => llm::ModelArchitecture::Gpt2,
        "gptj" => llm::ModelArchitecture::GptJ,
        "gpt-neox" => llm::ModelArchitecture::GptNeoX,
        "llama" => llm::ModelArchitecture::Llama,
        "mpt3" => llm::ModelArchitecture::Mpt,
        _ => panic!(), // TODO don't panic
    };

    llm::load_dynamic(
        model_type,
        weights_path,
        llm::ModelParameters::default(),
        None,
        |_| {},
    )
    .unwrap()
}
