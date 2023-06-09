use std::path::Path;

use error_stack::{IntoReport, Report, ResultExt};
use rust_bert::{pipelines::sentence_embeddings::SentenceEmbeddingsTokenizerOutput, RustBertError};

use super::{
    error::ModelError,
    rust_bert_sentence_embeddings::{
        create_model, SentenceEmbeddingsEncoder, SentenceEmbeddingsTokenizer,
    },
    ModelParams,
};

pub struct BiEncoderModel {
    pub name: String,
    tokenizer: SentenceEmbeddingsTokenizer,
    num_dimensions: i64,

    worker_thread: std::thread::JoinHandle<()>,
    worker_tx: flume::Sender<WorkerMessage>,
}

struct WorkerData {
    model: SentenceEmbeddingsEncoder,
}

pub type BiEncoderResult = Vec<Vec<f32>>;

enum WorkerMessage {
    Encode(WorkerEncodeMessage),
    Close,
}

struct WorkerEncodeMessage {
    tokenized: SentenceEmbeddingsTokenizerOutput,
    result: oneshot::Sender<Result<Vec<Vec<f32>>, Report<RustBertError>>>,
}

impl BiEncoderModel {
    pub fn new(
        name: String,
        params: &ModelParams,
        model_dir: Option<&Path>,
    ) -> Result<Self, Report<ModelError>> {
        let Some(model_dir) = model_dir else {
            return Err(ModelError::LoadingError).into_report()
                .attach_printable("Model directory not provided");
        };

        let (tokenizer, model) = create_model(model_dir)?;

        let (tx, rx) = flume::bounded(10);
        let num_dimensions = model.embeddings_dim;
        let worker_thread = std::thread::spawn(|| worker_thread(rx, model));

        Ok(Self {
            name,
            tokenizer,
            num_dimensions,
            worker_tx: tx,
            worker_thread,
        })
    }

    pub fn dimensions(&self) -> i64 {
        self.num_dimensions
    }

    pub fn encode<S: AsRef<str> + Sync>(
        &self,
        sentences: &[S],
    ) -> Result<BiEncoderResult, Report<ModelError>> {
        let tokenized = self.tokenizer.tokenize(sentences);
        let (tx, rx) = oneshot::channel();
        let msg = WorkerEncodeMessage {
            tokenized,
            result: tx,
        };

        self.worker_tx
            .send(WorkerMessage::Encode(msg))
            .map_err(|_| ModelError::WorkerClosed)
            .into_report()?;

        let result = rx
            .recv()
            .into_report()
            .change_context(ModelError::WorkerClosed)?;

        result.change_context(ModelError::ModelFailure)
    }
}

impl Drop for BiEncoderModel {
    fn drop(&mut self) {
        // TODO need a close with a timeout
        self.worker_tx.send(WorkerMessage::Close).ok();
        // self.worker_thread.join().ok();
    }
}

fn worker_thread(worker_rx: flume::Receiver<WorkerMessage>, model: SentenceEmbeddingsEncoder) {
    for msg in worker_rx {
        match msg {
            WorkerMessage::Encode(msg) => {
                let embeddings = model.encode(msg.tokenized);
                let vec = embeddings.and_then(|e| {
                    Vec::try_from(e.embeddings)
                        .map_err(RustBertError::from)
                        .into_report()
                });
                msg.result.send(vec).ok();
            }
            WorkerMessage::Close => break,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::models::{download::ModelCache, ModelLocation, ModelParams};

    #[test]
    fn test_model() {
        dotenvy::dotenv().ok();
        let cache = ModelCache::from_env().expect("Creating model cache");

        let params = ModelParams::RustBert(ModelLocation {
            location: "huggingface:sentence-transformers/all-MiniLM-L6-v2".to_string(),
        });

        let model_path = cache
            .download_if_needed(&params)
            .expect("Downloading model");
        let model = super::BiEncoderModel::new("test".to_string(), &params, model_path.as_deref())
            .expect("loading model");
        let encoding = model
            .encode(&["hello world", "goodbye world"])
            .expect("encoding");
        assert_eq!(encoding.len(), 2, "one encoding per input");
        assert_eq!(encoding[0].len(), 384, "length of encoding vector");
    }
}
