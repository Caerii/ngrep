use clap::error::ErrorKind;
use clap::CommandFactory;
use edit::get_editor;
use fancy_regex::{NeuralMatcherFactory, RegexBuilder};
use std::io::BufRead;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::config::{ModelConfig, NgrepConfig};
use crate::formatters::MatchFormatter;
use crate::neural_matchers::EmbedNeuralMatcherFactory;
use crate::Args;
use embeddings::converters::{converts, Formats};
use embeddings::ng;

pub fn handle_import<P: AsRef<Path>>(
    config: &mut NgrepConfig,
    path: P,
    name: &str,
    threshold: f64,
    default: bool,
) -> Result<()> {
    let file_name = path
        .as_ref()
        .file_name()
        .context("Error getting model file name")?;

    let model_path = PathBuf::from_iter([config.home(), file_name.into()]);
    let model_path = model_path.with_extension(ng::NG_EXTENSION);
    let model_conf = ModelConfig::new(name.into(), model_path, threshold)?;

    converts(Formats::Text, path.as_ref(), model_conf.path.as_ref())
        .context("Error during import of the model")?;

    config.add_model(&model_conf, default)
}

pub fn handle_config(config: &NgrepConfig) -> Result<()> {
    let editor = get_editor().context("No default $EDITOR found")?;

    let error = Command::new(editor).args([config.path()]).exec();

    eprintln!("Failed to exec: {}", error);
    std::process::exit(1);
}

pub fn handle_match(config: &mut NgrepConfig, args: Args, reader: Box<dyn BufRead>) -> Result<()> {
    let mut cli = Args::command();

    if args.pattern.is_none() {
        cli.error(
            ErrorKind::MissingRequiredArgument,
            "Missing required argument: pattern",
        )
        .exit();
    }

    let pattern_str = args.pattern.as_ref().unwrap();
    let model = config
        .model()
        .context("No default model found, run `ngrep import` first")?;
    let matcher_factory = EmbedNeuralMatcherFactory::new(&model.path, model.threshold)
        .context("Error during model initialization")?;

    let pattern = RegexBuilder::new(pattern_str)
        .neural_matcher_factory(&(Arc::new(matcher_factory) as Arc<dyn NeuralMatcherFactory>))
        .build()
        .context("Invalid regex pattern")?;

    let formatter = MatchFormatter::new(args.line_number, args.only_matching);
    for (line_inx, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let captures = pattern
            .find_iter(line.as_str())
            .map(|cap| cap.unwrap())
            .collect::<Vec<_>>();

        if !captures.is_empty() {
            formatter.display_line(line_inx, &line, &captures);
        }
    }

    Ok(())
}
