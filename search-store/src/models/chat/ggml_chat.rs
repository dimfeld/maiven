use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use error_stack::{IntoReport, Report, ResultExt};
use llm::{InferenceParameters, InferenceSessionConfig, OutputRequest};
use rayon::prelude::*;
use tracing::{info, instrument};

use crate::models::{completion::CompletionModel, ggml, ModelError};

use super::{ChatModel, ChatRole, ChatSubmission};

pub struct GgmlChatModel {
    name: String,
    model: Box<dyn llm::Model>,
    start_token: Option<llm::TokenId>,
    end_token: Option<llm::TokenId>,
    newline_token: Option<llm::TokenId>,
}

impl GgmlChatModel {
    pub fn new(
        name: String,
        model_type: &str,
        weights_path: &Path,
        tokenizer_path: Option<PathBuf>,
    ) -> Result<Self, Report<ModelError>> {
        let model = ggml::load_ggml_model(&name, model_type, weights_path, tokenizer_path)?;
        let vocab = model.vocabulary();
        let start_token = vocab.id("<|im_start|>".as_bytes());
        let end_token = vocab.id("<|im_end|>".as_bytes());
        let newline_token = vocab.tokenize("\n", false).unwrap();
        Ok(Self {
            name,
            model,
            start_token,
            end_token,
            newline_token: newline_token.get(0).map(|(_, token)| *token),
        })
    }
}

impl ChatModel for GgmlChatModel {
    #[instrument(skip(self), fields(name = %self.name))]
    fn chat(&self, submission: ChatSubmission) -> Result<super::ChatMessage, Report<ModelError>> {
        let vocab = self.model.vocabulary();

        let token_list = submission
            .messages
            .par_iter()
            .map(|message| vocab.tokenize(&message.content, false))
            .collect::<Result<Vec<_>, _>>()
            .into_report()
            .change_context(ModelError::ModelFailure)?;

        let padding_count = match (self.start_token.is_some(), self.end_token.is_some()) {
            (true, true) => 2,
            (true, false) | (false, true) => 1,
            (false, false) => 0,
        };

        let token_count: usize = token_list
            .iter()
            .map(|tokens| tokens.len() + padding_count + 1)
            .sum();

        let mut tokens = Vec::with_capacity(token_count);
        // if let Some(bot_token) = self.model.bot_token_id() {
        //     tokens.push(bot_token);
        // }

        for t in token_list {
            if let Some(start_token) = self.start_token {
                tokens.push(start_token);
            }

            tokens.extend(t.iter().map(|(_, token)| token));

            if let Some(end_token) = self.end_token {
                tokens.push(end_token);
            }

            if let Some(newline_token) = self.newline_token {
                tokens.push(newline_token);
            }
        }

        let temperature = submission.temperature.unwrap_or(0.3);

        let params = InferenceParameters {
            sampler: Arc::new(llm::samplers::TopPTopK {
                temperature,
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut output_tokens = Vec::new();

        let mut session = self.model.start_session(InferenceSessionConfig::default());
        let mut output = OutputRequest {
            all_logits: None,
            embeddings: None,
        };
        self.model
            .evaluate(&mut session, &params, &tokens, &mut output);
        info!(input_tokens=%tokens.len(), "Evaluated input");

        let mut num_output_tokens = 0;
        while let Ok(token) = session.infer_next_token(
            self.model.as_ref(),
            &params,
            &mut output,
            &mut rand::thread_rng(),
        ) {
            num_output_tokens += 1;
            if token != "<|im_end|>".as_bytes() && token != "<|im_start|>".as_bytes() {
                output_tokens.extend(token);
            }
        }

        info!(output_tokens=%num_output_tokens, "Done");

        Ok(super::ChatMessage {
            role: ChatRole::Assistant,
            content: String::from_utf8_lossy(&output_tokens).to_string(),
            name: None,
        })
    }
}

impl CompletionModel for GgmlChatModel {
    #[instrument(skip(self), fields(name = %self.name))]
    fn complete(
        &self,
        submission: crate::models::completion::CompletionSubmission,
    ) -> Result<String, Report<ModelError>> {
        self.model.complete(submission)
    }
}
