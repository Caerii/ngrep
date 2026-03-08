use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::{ParseFloatError, ParseIntError};
use std::path::{Path, PathBuf};
use std::result;

use crate::ng::{self, WordEmbedding};
use anyhow::{bail, Context, Result};
use tqdm::tqdm;

pub enum Formats {
    Text,
}

trait EmbedAdapter {
    fn embeddings(&self) -> Result<(usize, Box<dyn Iterator<Item = Result<WordEmbedding>> + '_>)>;
}

pub fn to_ng<P: AsRef<Path>>(format: Formats, input: P, output: P) -> Result<()> {
    let model = match format {
        Formats::Text => Box::new(TextEmbeddings::new(input.as_ref().into())),
    };
    let (count, embeddings) = model.embeddings()?;

    let embeddings = tqdm(embeddings).desc(Some("import")).total(Some(count));
    ng::to_file(output, embeddings)
}

// Embeddings Formats ---------------------------------------------------------------------------
struct TextEmbeddings {
    input: PathBuf,
}

impl TextEmbeddings {
    fn new(input: PathBuf) -> Self {
        TextEmbeddings { input }
    }

    fn parse_header(&self, header: &str) -> Result<(usize, usize)> {
        let parts = header
            .split_whitespace()
            .map(|part| part.parse::<usize>())
            .collect::<std::result::Result<Vec<usize>, ParseIntError>>()
            .context("Invalid header")?;

        if parts.len() != 2 {
            bail!("Invalid header")
        }

        Ok((parts[0], parts[1]))
    }

    fn parse_line(&self, line: &str, dim: usize) -> Result<WordEmbedding> {
        let parts: Vec<&str> = line.trim_end().split(" ").collect();

        let token = parts[0].to_string();
        let embed = parts[1..]
            .iter()
            .map(|elem| elem.parse::<f32>())
            .collect::<result::Result<Vec<f32>, ParseFloatError>>()
            .context("Invalid embedding line")?;

        if embed.len() != dim {
            bail!("Invalid number of entries");
        }

        Ok((token, embed))
    }
}

impl EmbedAdapter for TextEmbeddings {
    fn embeddings(&self) -> Result<(usize, Box<dyn Iterator<Item = Result<WordEmbedding>> + '_>)> {
        let model = BufReader::new(File::open(&self.input)?);
        let mut lines = model.lines();

        let header = lines.next().context("Missing header")??;

        let (count, dim) = self.parse_header(&header)?;

        let iter = lines.map(move |line| {
            let line = line?;
            self.parse_line(&line, dim)
        });

        Ok((count, Box::new(iter)))
    }
}
