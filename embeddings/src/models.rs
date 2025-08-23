use candle_core::{Device, Tensor};
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;

use crate::ng::NgStorage;
use anyhow::Result;

pub trait Embed: Debug {
    fn embed(&self, text: &str) -> Result<Tensor, ()>;
    fn has_prefix(&self, prefix: &str) -> bool;
}

pub struct EmbeddingLoader;

impl EmbeddingLoader {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Arc<dyn Embed>> {
        let path_buf = path.as_ref().to_path_buf();
        let storage = NgStorage::from_file(path_buf)?;

        Ok(Arc::new(LookupEmbeddingModel::load(storage, Device::Cpu)))
    }
}

// Embeddings Models ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct LookupEmbeddingModel {
    device: Device,
    storage: NgStorage,
}

impl LookupEmbeddingModel {
    pub fn load(storage: NgStorage, device: Device) -> Self {
        Self { device, storage }
    }
}

impl Embed for LookupEmbeddingModel {
    fn embed(&self, token: &str) -> Result<Tensor, ()> {
        match self.storage.vector(token) {
            Ok(embed) => {
                let dim = embed.len();
                let tensor = Tensor::from_vec(embed, (1, dim), &self.device).unwrap();
                Ok(tensor.unsqueeze(0).unwrap())
            }
            Err(..) => Err(()),
        }
    }

    fn has_prefix(&self, prefix: &str) -> bool {
        self.storage.header.keys.has_prefix(prefix)
    }
}
