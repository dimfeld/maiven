use std::{path::Path, sync::Arc};

use error_stack::{IntoReport, Report, ResultExt};
use llm::{InferenceParameters, InferenceSessionConfig, OutputRequest};
use tracing::{info, instrument};

use crate::models::{ggml, ModelError};

use super::CompletionModel;

pub struct GgmlCompletionModel {
    name: String,
    model: Box<dyn llm::Model>,
}

impl GgmlCompletionModel {
    pub fn new(
        name: String,
        model_type: &str,
        weights_path: &Path,
    ) -> Result<Self, Report<ModelError>> {
        let model = ggml::load_ggml_model(model_type, weights_path)?;
        Ok(Self { name, model })
    }
}

impl CompletionModel for GgmlCompletionModel {
    #[instrument(skip(self), fields(name = %self.name))]
    fn complete(
        &self,
        submission: super::CompletionSubmission,
    ) -> Result<String, Report<ModelError>> {
        self.model.complete(submission)
    }
}

impl CompletionModel for dyn llm::Model {
    fn complete(
        &self,
        submission: super::CompletionSubmission,
    ) -> Result<String, Report<ModelError>> {
        let tokens = self
            .vocabulary()
            .tokenize(&submission.prompt, true)
            .into_report()
            .change_context(ModelError::ModelFailure)?
            .into_iter()
            .map(|(_, token)| token)
            .collect::<Vec<_>>();

        let mut session = self.start_session(InferenceSessionConfig::default());
        let mut output = OutputRequest::default();

        let temperature = submission.temperature.unwrap_or(0.3);
        let params = InferenceParameters {
            sampler: Arc::new(llm::samplers::TopPTopK {
                temperature,
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut output_tokens = Vec::new();

        self.evaluate(&mut session, &params, &tokens, &mut output);
        info!(input_tokens=%tokens.len(), "Evaluated input");

        let mut num_output_tokens = 0;
        while let Ok(token) =
            session.infer_next_token(self, &params, &mut output, &mut rand::thread_rng())
        {
            output_tokens.extend(token);
            num_output_tokens += 1;
        }
        info!(output_tokens=%num_output_tokens, "Done");

        Ok(String::from_utf8_lossy(&output_tokens).to_string())
    }
}
