use candle_core::Tensor;

pub mod converters;
pub mod matchers;
pub mod models;
pub mod ng;

pub type Embedding = Tensor;
