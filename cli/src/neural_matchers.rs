use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use embeddings::matchers::{CosineMatcher, Match};
use embeddings::models::{Embed, EmbeddingLoader};
use embeddings::Embedding;

use anyhow::Result;
use fancy_regex::{NeuralExpr, NeuralMatcher, NeuralMatcherFactory};

/// A factory that creates an EmbedNeuralMatcher (impl NeuralMatcher) for a NeuralExpr.
///
/// Instantiates a NeuralMatcher able to match a given string (i.e value of NeuralExpr)
/// using an Embed model and a matcher strategy (e.g: cosine distance)
#[derive(Debug)]
pub struct EmbedNeuralMatcherFactory {
    model: Arc<dyn Embed>,
    threshold: f64,
}

impl EmbedNeuralMatcherFactory {
    pub fn new(model_path: &PathBuf, threshold: f64) -> Result<Self> {
        Ok(EmbedNeuralMatcherFactory {
            model: EmbeddingLoader::load(model_path)?,
            threshold,
        })
    }
}

impl NeuralMatcherFactory for EmbedNeuralMatcherFactory {
    fn matcher_for(&self, expr: &NeuralExpr) -> Result<Arc<dyn NeuralMatcher>, io::Error> {
        let expr_value = expr.value.clone();
        let expr_threshold = expr.threshold.or(Some(self.threshold)).unwrap();

        let matcher = EmbedNeuralMatcher::new(self.model.clone(), expr_value, expr_threshold);

        Ok(Arc::new(matcher))
    }
}

/// A NeuralMatcher based on a Word Embedding model and a vector-based distance metric
///
/// Combines an Embed model, a Match strategy (e.g: cosine distance),
/// to creates a matcher linked to a specific value String.
#[derive(Debug)]
struct EmbedNeuralMatcher {
    model: Arc<dyn Embed>,
    matcher: Box<dyn Match>,
    value: Embedding,
}

impl EmbedNeuralMatcher {
    fn new(model: Arc<dyn Embed>, value: String, threshold: f64) -> Self {
        let value = model.embed(&value).unwrap();

        EmbedNeuralMatcher {
            model,
            value,
            matcher: Box::new(CosineMatcher::new(threshold)),
        }
    }
}

impl NeuralMatcher for EmbedNeuralMatcher {
    fn matches(&self, text: &str) -> bool {
        match self.model.embed(text) {
            Ok(text_embed) => self.matcher.is_match(&self.value, &text_embed).unwrap(),
            Err(_) => return false,
        }
    }

    fn might_match(&self, text: &str) -> bool {
        self.model.has_prefix(text)
    }
}
