use std::fmt::Debug;

use anyhow::{bail, Context, Result};
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
        let num = num.to_dtype(DType::F64)?.to_scalar::<f64>()?;

        let den = (self.norm(lhs)? * self.norm(rhs)?)?;
        let den = den.to_dtype(DType::F64)?.to_scalar::<f64>()?;
        if den == 0.0 {
            bail!("Cannot compute cosine similarity on a zero-vector");
        }

        Ok(num / den)
    }

    fn norm(&self, tensor: &Tensor) -> Result<Tensor> {
        (tensor * tensor)?
            .sum_all()?
            .sqrt()
            .context("Error computing norm")
    }
}

impl Match for CosineMatcher {
    fn is_match(&self, lhs: &Tensor, rhs: &Tensor) -> Result<bool> {
        let sim = self.cosine_sim(lhs, rhs)?;
        Ok(sim > self.0)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use candle_core::Device;

    fn setup_matcher(threshold: f64) -> CosineMatcher {
        CosineMatcher::new(threshold)
    }

    #[test]
    fn test_cosine_similarity() {
        let lhs = Tensor::new(&[[0.0, 1.0, 2.0]], &Device::Cpu).unwrap();
        let rhs = Tensor::new(&[[2.0, 1.0, 0.0]], &Device::Cpu).unwrap();

        let matcher = setup_matcher(0.0);
        let actual = matcher.cosine_sim(&lhs, &rhs).unwrap();
        let expected = 0.2;

        assert!((actual - expected).abs() < 1e-5);
    }

    #[test]
    fn test_cosine_matcher_threshold() {
        let lhs = Tensor::new(&[[0.0, 1.0, 2.0]], &Device::Cpu).unwrap();
        let rhs = Tensor::new(&[[2.0, 1.0, 0.0]], &Device::Cpu).unwrap();

        let high_threshold_matcher = setup_matcher(0.5);
        assert!(!high_threshold_matcher.is_match(&lhs, &rhs).unwrap());

        let low_threshold_matcher = setup_matcher(0.19);
        assert!(low_threshold_matcher.is_match(&lhs, &rhs).unwrap());
    }

    #[test]
    #[should_panic(expected = "Cannot compute cosine similarity on a zero-vector")]
    fn test_zero_vector_panics() {
        let lhs = Tensor::new(&[[0.0, 0.0, 0.0]], &Device::Cpu).unwrap();
        let rhs = Tensor::new(&[[1.0, 2.0, 3.0]], &Device::Cpu).unwrap();

        let matcher = setup_matcher(0.5);
        matcher.cosine_sim(&lhs, &rhs).unwrap();
    }
}
