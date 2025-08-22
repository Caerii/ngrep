use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use embeddings::matchers::{CosineMatcher, Match};
use embeddings::models::{Embed, EmbeddingLoader};
use embeddings::Embedding;

use anyhow::Result;
use fancy_regex::{NeuralExpr, NeuralMatcher, NeuralMatcherFactory};

#[derive(Debug)]
pub struct EmbedNeuralMatcherFactory {
    model: Arc<dyn Embed>,
    threshold: f64,
}

impl EmbedNeuralMatcherFactory {
    pub fn new(model_path: &PathBuf, threshold: f64) -> Result<Self> {
        let model = EmbeddingLoader::load(model_path)?;
        Ok(Self { model, threshold })
    }
}

impl NeuralMatcherFactory for EmbedNeuralMatcherFactory {
    fn matcher_for(&self, expr: &NeuralExpr) -> Result<Arc<dyn NeuralMatcher>, io::Error> {
        let expr_value = &expr.value;
        let expr_threshold = expr.threshold.unwrap_or(self.threshold);

        let matcher = EmbedNeuralMatcher::new(self.model.clone(), expr_value, expr_threshold);

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
    fn new(model: Arc<dyn Embed>, value: &str, threshold: f64) -> Self {
        let embedding = model
            .embed(value)
            .expect(&format!("Failed to embed value: {}", value));

        let matcher = Box::new(CosineMatcher::new(threshold));

        Self {
            model,
            embedding,
            matcher,
        }
    }
}

impl NeuralMatcher for EmbedNeuralMatcher {
    fn is_match(&self, text: &str) -> bool {
        match self.model.embed(text) {
            Ok(text_embed) => self.matcher.is_match(&self.embedding, &text_embed).unwrap(),
            Err(_) => return false,
        }
    }

    fn might_match(&self, text: &str) -> bool {
        self.model.has_prefix(text)
    }
}
