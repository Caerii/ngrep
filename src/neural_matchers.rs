use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use embeddings::matchers::{CosineMatcher, Match};
use embeddings::models::{Embed, EmbeddingLoader};
use embeddings::Embedding;

use anyhow::Result;
use fancy_regex::{NeuralExpr, NeuralMatcher, NeuralMatcherFactory};

#[derive(Debug)]
pub struct EmbedNeuralMatcherFactory {
    model_path: PathBuf,
    threshold: f64,
    model: Option<Arc<dyn Embed>>,
}

impl EmbedNeuralMatcherFactory {
    pub fn new<P: AsRef<Path>>(model_path: P, threshold: f64) -> Self {
        let model_path = model_path.as_ref().to_path_buf();

        Self {
            model_path,
            threshold,
            model: None,
        }
    }
}

impl NeuralMatcherFactory for EmbedNeuralMatcherFactory {
    fn initialize(&mut self) -> Result<(), Error> {
        match EmbeddingLoader::load(&self.model_path) {
            Ok(model) => {
                self.model = Some(model);
                Ok(())
            }
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("Error loading model: {}", e),
            )),
        }
    }

    fn matcher_for(&self, expr: &NeuralExpr) -> Result<Arc<dyn NeuralMatcher>, Error> {
        let value = &expr.value;
        let threshold = expr.threshold.unwrap_or(self.threshold);
        let model = self.model.as_ref().ok_or_else(|| {
            Error::new(
                ErrorKind::Other,
                "Model not initialized. Call initialize() first.",
            )
        })?;

        let matcher = EmbedNeuralMatcher::new(model.clone(), value, threshold)?;
        Ok(Arc::new(matcher))
    }
}

#[derive(Debug)]
struct EmbedNeuralMatcher {
    model: Arc<dyn Embed>,
    matcher: Box<dyn Match>,
    embedding: Embedding,
}

impl EmbedNeuralMatcher {
    fn build_query_embedding(model: &dyn Embed, value: &str) -> Result<Embedding, Error> {
        let chunks: Vec<&str> = value.split("::").map(str::trim).collect();
        if chunks.is_empty() || chunks.iter().any(|chunk| chunk.is_empty()) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Invalid neural query '{value}'"),
            ));
        }

        let mut query: Option<Embedding> = None;
        for chunk in &chunks {
            let embedding = model.embed(chunk).map_err(|err| {
                Error::new(
                    ErrorKind::InvalidInput,
                    format!("Can't embed value '{}': {err:?}", chunk),
                )
            })?;

            query = Some(match query {
                Some(acc) => (&acc + &embedding).map_err(|err| {
                    Error::new(
                        ErrorKind::Other,
                        format!("Error combining neural query '{value}': {err}"),
                    )
                })?,
                None => embedding,
            });
        }

        let query = query.expect("query chunks are never empty");
        if chunks.len() > 1 {
            query.affine(1.0 / chunks.len() as f64, 0.0).map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Error averaging neural query '{value}': {err}"),
                )
            })
        } else {
            Ok(query)
        }
    }

    fn new(model: Arc<dyn Embed>, value: &str, threshold: f64) -> Result<Self, Error> {
        let embedding = Self::build_query_embedding(model.as_ref(), value)?;

        let matcher = Box::new(CosineMatcher::new(threshold));

        Ok(Self {
            model,
            embedding,
            matcher,
        })
    }
}

impl NeuralMatcher for EmbedNeuralMatcher {
    fn is_match(&self, text: &str) -> bool {
        match self.model.embed(text) {
            Ok(text_embed) => self
                .matcher
                .is_match(&self.embedding, &text_embed)
                .unwrap_or(false),
            Err(_) => return false,
        }
    }

    fn might_match(&self, text: &str) -> bool {
        self.model.has_prefix(text)
    }

    fn max_codepoints(&self) -> usize {
        32
    }
}
