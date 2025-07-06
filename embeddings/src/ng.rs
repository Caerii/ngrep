use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use trie_rs::Trie;

const NG_EXTENSION: &str = "ng";
const NG_MAGIC: &str = "NG";
const NG_VERSION: u8 = 0x0;

pub type WordEmbedding = (String, Vec<f32>);

pub fn to_file<P: AsRef<Path>>(
    output: P,
    embeddings: impl Iterator<Item = Result<WordEmbedding>>,
) -> Result<()> {
    let output = output.as_ref().with_extension(NG_EXTENSION);
    let mut writer = BufWriter::new(File::create(output)?);

    let mut embeddings = embeddings.peekable();
    let we_0 = match &embeddings.peek() {
        Some(Ok(we)) => we.clone(),
        _ => bail!("No embeddings found"),
    };

    let we_dim: u32 = we_0.1.len().try_into()?;

    // magic & version
    writer.write(NG_MAGIC.as_bytes())?;
    writer.write(&[NG_VERSION])?;

    // header
    let (we_tokens, we_vectors): (Vec<String>, Vec<Vec<f32>>) = embeddings
        .collect::<Result<Vec<WordEmbedding>>>()?
        .into_iter()
        .unzip();
    let we_tokens = Vocab::from_array(&we_tokens);

    let we_count: u32 = we_vectors.len().try_into()?;

    let header = NgHeader {
        count: we_count,
        dim: we_dim,
        keys: we_tokens,
    };

    bincode::serde::encode_into_std_write(&header, &mut writer, bincode::config::standard())?;

    // raw embeddings
    for vector in we_vectors {
        let vector_bytes = vector
            .iter()
            .flat_map(|e| e.to_le_bytes())
            .collect::<Vec<u8>>();

        writer.write_all(&vector_bytes)?;
    }

    Ok(())
}

#[derive(Debug)]
pub struct NgStorage {
    pub header: NgHeader,
    mmap: Mmap,

    vector_seek: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NgHeader {
    pub count: u32,
    pub dim: u32,
    pub keys: Vocab,
}

impl NgStorage {
    pub fn from_file(input: PathBuf) -> Result<NgStorage> {
        let file = File::open(input)?;

        let mut reader = BufReader::new(&file);
        let mut buf = Vec::<u8>::new();

        // magic
        buf.resize(NG_MAGIC.len(), 0x00);
        reader.read_exact(&mut buf)?;

        if buf != NG_MAGIC.as_bytes() {
            bail!("Format unknown")
        }

        // version
        buf.resize(1, 0x00);
        reader.read_exact(&mut buf)?;

        if buf[0] != NG_VERSION {
            bail!("Unsupported ng version '{}' found", buf[0])
        }

        // header
        let header: NgHeader =
            bincode::serde::decode_from_reader(&mut reader, bincode::config::standard())?;

        // mmap
        let mmap = unsafe { Mmap::map(&file)? };
        let pos: usize = reader.stream_position()?.try_into()?;

        Ok(NgStorage {
            header,
            mmap,
            vector_seek: pos,
        })
    }

    pub fn vector(&self, token: &str) -> Result<Vec<f32>> {
        let inx = self.header.keys.inx(token);
        match inx {
            None => bail!("Vector '{}' not found", token),
            Some(inx) => self.vector_at(inx),
        }
    }

    fn vector_at(&self, inx: usize) -> Result<Vec<f32>> {
        let line_size = (self.header.dim * 4) as usize;
        let line_start = self.vector_seek + (line_size * inx);

        let vector = decode_le_bytes_to_vec::<f32>(&self.mmap[line_start..line_start + line_size])?;

        Ok(vector)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vocab {
    vocab: HashMap<String, usize>,
    trie: Trie<u8>,
}

impl Vocab {
    fn from_array(array: &[String]) -> Self {
        let vocab: HashMap<String, usize> = array
            .iter()
            .enumerate()
            .map(|(i, t)| (t.clone(), i))
            .collect();

        Self {
            vocab,
            trie: Trie::from_iter(array),
        }
    }

    pub fn inx(&self, token: &str) -> Option<usize> {
        self.vocab.get(token).map(|&i| i)
    }

    pub fn has_prefixes(&self, prefix: &str) -> bool {
        if self.trie.exact_match(prefix) {
            return true;
        }
        let next: Option<String> = self.trie.postfix_search(prefix).next();
        next.is_some()
    }
}

// Parsing Utils -----------------------------------------------------------------------------------

trait FromLeBytes: Sized {
    const BYTES: usize;

    fn from_le_bytes(bytes: &[u8]) -> Result<Self>;
}

impl FromLeBytes for f32 {
    const BYTES: usize = 4;

    fn from_le_bytes(bytes: &[u8]) -> Result<Self> {
        let chunk: [u8; Self::BYTES] = bytes.try_into()?;

        Ok(f32::from_le_bytes(chunk))
    }
}

fn decode_le_bytes_to_vec<T: FromLeBytes>(bytes: &[u8]) -> Result<Vec<T>> {
    if bytes.len() % T::BYTES != 0 {
        bail!("Can't decode bytes")
    }

    bytes
        .chunks_exact(T::BYTES)
        .map(|chunk| T::from_le_bytes(chunk))
        .collect()
}
