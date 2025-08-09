use std::fmt::Debug;

use anyhow::{bail, Result};
use candle_core::{DType, Tensor};

pub trait Match: Debug {
    fn is_match(&self, lhs: &Tensor, rhs: &Tensor) -> Result<bool>;
}

#[derive(Debug)]
pub struct CosineMatcher(f64);

impl CosineMatcher {
    pub fn new(threshold: f64) -> Self {
        CosineMatcher(threshold)
    }

    fn cosine_sim(&self, lhs: &Tensor, rhs: &Tensor) -> Result<f64> {
        let num = lhs.matmul(&rhs.t()?)?.flatten_all()?.squeeze(0)?;
        let den_tensor = (self.norm(lhs)? * self.norm(rhs)?)?;
        let den = den_tensor.to_dtype(DType::F64)?.to_scalar::<f64>()?;

        if den == 0.0 {
            bail!("Cannot compute cosine similarity on a zero-vector");
        }

        let num = num.to_dtype(DType::F64)?.to_scalar::<f64>()?;
        Ok(num / den)
    }

    fn norm(&self, tensor: &Tensor) -> Result<Tensor> {
        Ok((tensor * tensor)?.sum_all()?.sqrt()?)
    }
}

impl Match for CosineMatcher {
    fn is_match(&self, lhs: &Tensor, rhs: &Tensor) -> Result<bool> {
        let sim = self.cosine_sim(lhs, rhs)?;
        Ok(sim > self.0)
    }
}
