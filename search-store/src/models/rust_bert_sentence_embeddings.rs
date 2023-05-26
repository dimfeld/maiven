use std::{borrow::Cow, path::Path};

use error_stack::{IntoReport, Result, ResultExt};
use rust_bert::{
    pipelines::{
        common::{ConfigOption, ModelType, TokenizerOption},
        sentence_embeddings::{
            layers::{Dense, DenseConfig, Pooling, PoolingConfig},
            SentenceEmbeddingsConfig, SentenceEmbeddingsModelOutput,
            SentenceEmbeddingsModulesConfig, SentenceEmbeddingsOption,
            SentenceEmbeddingsSentenceBertConfig, SentenceEmbeddingsTokenizerConfig,
            SentenceEmbeddingsTokenizerOutput,
        },
    },
    resources::{LocalResource, ResourceProvider},
    Config, RustBertError,
};
use rust_tokenizers::{tokenizer::TruncationStrategy, TokenizedInput};
use tch::{nn, Tensor};

use super::{transformers::read_model_config, ModelError};

pub struct SentenceEmbeddingsTokenizer {
    sentence_bert_config: SentenceEmbeddingsSentenceBertConfig,
    tokenizer: TokenizerOption,
    truncation_strategy: TruncationStrategy,
}

impl SentenceEmbeddingsTokenizer {
    pub fn new(
        sentence_bert_config: SentenceEmbeddingsSentenceBertConfig,
        tokenizer: TokenizerOption,
        truncation_strategy: TruncationStrategy,
    ) -> Self {
        Self {
            sentence_bert_config,
            tokenizer,
            truncation_strategy,
        }
    }

    pub fn generate_token_tensors(
        &self,
        token_ids: &[TokenizedInput],
    ) -> SentenceEmbeddingsTokenizerOutput {
        let max_len = token_ids
            .iter()
            .map(|input| input.token_ids.len())
            .max()
            .unwrap_or(0);

        let pad_token_id = self.tokenizer.get_pad_id().unwrap_or(0);

        // Pad any vectors shorter than the max length so that they are all the same.
        let tokens_ids = token_ids
            .iter()
            .map(|input| {
                let input = input.token_ids.as_slice();
                if input.len() == max_len {
                    Cow::Borrowed(input)
                } else {
                    let mut padded = Vec::with_capacity(max_len);
                    padded.extend_from_slice(input);
                    padded.resize(max_len, pad_token_id);
                    Cow::Owned(padded)
                }
            })
            .collect::<Vec<_>>();

        let tokens_masks = tokens_ids
            .iter()
            .map(|input| {
                Tensor::from_slice(
                    &input
                        .iter()
                        .map(|&e| i64::from(e != pad_token_id))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        let tokens_ids = tokens_ids
            .into_iter()
            .map(|input| Tensor::from_slice(&(input)))
            .collect::<Vec<_>>();

        SentenceEmbeddingsTokenizerOutput {
            tokens_ids,
            tokens_masks,
        }
    }

    /// Tokenizes the inputs
    pub fn tokenize<S>(&self, inputs: &[S]) -> SentenceEmbeddingsTokenizerOutput
    where
        S: AsRef<str> + Sync,
    {
        let tokenized_input = self.tokenizer.encode_list(
            inputs,
            self.sentence_bert_config.max_seq_length,
            &self.truncation_strategy,
            0,
        );

        self.generate_token_tensors(&tokenized_input)
    }
}

pub struct SentenceEmbeddingsEncoder {
    pub var_store: nn::VarStore,
    pub transformer: SentenceEmbeddingsOption,
    pub pooling_layer: Pooling,
    pub dense_layer: Option<Dense>,
    pub normalize_embeddings: bool,
    pub embeddings_dim: i64,
}

impl SentenceEmbeddingsEncoder {
    pub fn encode(
        &self,
        inputs: SentenceEmbeddingsTokenizerOutput,
    ) -> Result<SentenceEmbeddingsModelOutput, RustBertError> {
        let SentenceEmbeddingsTokenizerOutput {
            tokens_ids,
            tokens_masks,
        } = inputs;
        let tokens_ids = Tensor::stack(&tokens_ids, 0).to(self.var_store.device());
        let tokens_masks = Tensor::stack(&tokens_masks, 0).to(self.var_store.device());

        let (tokens_embeddings, all_attentions) =
            tch::no_grad(|| self.transformer.forward(&tokens_ids, &tokens_masks))?;

        let mean_pool =
            tch::no_grad(|| self.pooling_layer.forward(tokens_embeddings, &tokens_masks));
        let maybe_linear = if let Some(dense_layer) = &self.dense_layer {
            tch::no_grad(|| dense_layer.forward(&mean_pool))
        } else {
            mean_pool
        };
        let maybe_normalized = if self.normalize_embeddings {
            let norm = &maybe_linear
                .norm_scalaropt_dim(2, [1], true)
                .clamp_min(1e-12)
                .expand_as(&maybe_linear);
            maybe_linear / norm
        } else {
            maybe_linear
        };

        Ok(SentenceEmbeddingsModelOutput {
            embeddings: maybe_normalized,
            all_attentions,
        })
    }
}

fn model_from_config(
    config: SentenceEmbeddingsConfig,
) -> Result<(SentenceEmbeddingsTokenizer, SentenceEmbeddingsEncoder), RustBertError> {
    let SentenceEmbeddingsConfig {
        modules_config_resource,
        sentence_bert_config_resource,
        tokenizer_config_resource,
        tokenizer_vocab_resource,
        tokenizer_merges_resource,
        transformer_type,
        transformer_config_resource,
        transformer_weights_resource,
        pooling_config_resource,
        dense_config_resource,
        dense_weights_resource,
        device,
    } = config;

    let modules =
        SentenceEmbeddingsModulesConfig::from_file(modules_config_resource.get_local_path()?)
            .validate()?;

    // Setup tokenizer

    let tokenizer_config =
        SentenceEmbeddingsTokenizerConfig::from_file(tokenizer_config_resource.get_local_path()?);
    let sentence_bert_config = SentenceEmbeddingsSentenceBertConfig::from_file(
        sentence_bert_config_resource.get_local_path()?,
    );
    let tokenizer = TokenizerOption::from_file(
        transformer_type,
        tokenizer_vocab_resource
            .get_local_path()?
            .to_string_lossy()
            .as_ref(),
        tokenizer_merges_resource
            .as_ref()
            .map(|resource| resource.get_local_path())
            .transpose()?
            .map(|path| path.to_string_lossy().into_owned())
            .as_deref(),
        tokenizer_config
            .do_lower_case
            .unwrap_or(sentence_bert_config.do_lower_case),
        tokenizer_config.strip_accents,
        tokenizer_config.add_prefix_space,
    )?;

    // Setup transformer

    let mut var_store = nn::VarStore::new(tch::Device::cuda_if_available());
    let transformer_config = ConfigOption::from_file(
        transformer_type,
        transformer_config_resource.get_local_path()?,
    );
    let transformer =
        SentenceEmbeddingsOption::new(transformer_type, var_store.root(), &transformer_config)?;
    var_store
        .load(transformer_weights_resource.get_local_path()?)
        .map_err(|e| RustBertError::TchError(e.to_string()))?;

    // If on M1, switch to the GPU.
    // This needs to be done here instead of during the initial load, since the pretrained
    // models can't be loaded directly.
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    var_store.set_device(tch::Device::Mps);

    // Setup pooling layer

    let pooling_config = PoolingConfig::from_file(pooling_config_resource.get_local_path()?);
    let mut embeddings_dim = pooling_config.word_embedding_dimension;
    let pooling_layer = Pooling::new(pooling_config);

    // Setup dense layer

    let dense_layer = if modules.dense_module().is_some() {
        let dense_config = DenseConfig::from_file(dense_config_resource.unwrap().get_local_path()?);
        embeddings_dim = dense_config.out_features;
        Some(Dense::new(
            dense_config,
            dense_weights_resource.unwrap().get_local_path()?,
            device,
        )?)
    } else {
        None
    };

    let normalize_embeddings = modules.has_normalization();

    let tokenizer = SentenceEmbeddingsTokenizer {
        sentence_bert_config,
        tokenizer,
        truncation_strategy: TruncationStrategy::LongestFirst,
    };

    let encoder = SentenceEmbeddingsEncoder {
        var_store,
        transformer,
        pooling_layer,
        dense_layer,
        normalize_embeddings,
        embeddings_dim,
    };

    Ok((tokenizer, encoder))
}

pub fn create_model(
    model_path: &Path,
) -> Result<(SentenceEmbeddingsTokenizer, SentenceEmbeddingsEncoder), ModelError> {
    let config = model_config(model_path)?;
    model_from_config(config).change_context(ModelError::LoadingError)
}

fn local_resource(base_path: &Path, file: &str) -> Box<dyn ResourceProvider + Send> {
    Box::new(LocalResource::from(base_path.join(file)))
}

fn get_model_type(base_path: &Path) -> Result<ModelType, ModelError> {
    let model_type = read_model_config(base_path)
        .into_report()
        .change_context(ModelError::LoadingError)?;

    let transformer_type = match model_type.as_str() {
        "albert" => ModelType::Albert,
        "bart" => ModelType::Bart,
        "bert" => ModelType::Bert,
        "deberta" => ModelType::Deberta,
        "deberta-v2" => ModelType::DebertaV2,
        "distilbert" => ModelType::DistilBert,
        "electra" => ModelType::Electra,
        "fnet" => ModelType::FNet,
        "gpt2" => ModelType::GPT2,
        "gpt_neo" => ModelType::GPTNeo,
        "longformer" => ModelType::Longformer,
        "m2m_100" => ModelType::M2M100,
        "marian" => ModelType::Marian,
        "mbart" => ModelType::MBart,
        "mobilebert" => ModelType::MobileBert,
        "openai-gpt" => ModelType::OpenAiGpt,
        "pegasus" => ModelType::Pegasus,
        "prophetnet" => ModelType::ProphetNet,
        "reformer" => ModelType::Reformer,
        "roberta" => ModelType::Roberta,
        "t5" => ModelType::T5,
        "xlm-roberta" => ModelType::XLMRoberta,
        "xlnet" => ModelType::XLNet,
        _ => return Err(ModelError::UnknownModelType(model_type)).into_report(),
    };

    Ok(transformer_type)
}

fn model_config(path: &Path) -> Result<SentenceEmbeddingsConfig, ModelError> {
    let lr = |file: &str| local_resource(path, file);

    let transformer_type = get_model_type(path)?;

    let has_dense = path.join("2_Dense").exists();
    let has_merges = path.join("merges.txt").exists();

    Ok(SentenceEmbeddingsConfig {
        modules_config_resource: lr("modules.json"),
        transformer_type,
        transformer_config_resource: lr("config.json"),
        transformer_weights_resource: lr("rust_model.ot"),
        pooling_config_resource: lr("1_Pooling/config.json"),
        dense_config_resource: has_dense.then(|| lr("2_Dense/config.json")),
        dense_weights_resource: has_dense.then(|| lr("2_Dense/rust_model.ot")),
        sentence_bert_config_resource: lr("sentence_bert_config.json"),
        tokenizer_config_resource: lr("tokenizer_config.json"),
        tokenizer_vocab_resource: lr("vocab.txt"),
        tokenizer_merges_resource: has_merges.then(|| lr("merges.txt")),
        device: tch::Device::cuda_if_available(),
    })
}
