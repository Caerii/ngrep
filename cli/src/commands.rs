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

use crate::config::NgrepConfig;
use crate::formatters::MatchFormatter;
use crate::neural_matchers::EmbedNeuralMatcherFactory;
use crate::Args;
use embeddings::converters::{converts, Formats};
use embeddings::ng;

pub fn handle_import<P: AsRef<Path>>(
    config: &mut NgrepConfig,
    model_path: P,
    name: &str,
) -> Result<()> {
    let model_name: String = model_path
        .as_ref()
        .file_name()
        .context("Error getting model filename")?
        .to_string_lossy()
        .into();

    let output = PathBuf::from_iter([config.home(), model_name.into()]);
    let output = output.with_extension(ng::NG_EXTENSION);

    converts(Formats::Text, model_path.as_ref(), output.as_ref())
        .context("Error during import of the model")?;

    config.add_model(name, &output.to_string_lossy())
}

pub fn handle_config(config: &NgrepConfig) -> Result<()> {
    let editor = get_editor().context("No default editor found")?;

    let error = Command::new(editor).args([config.path()]).exec();

    eprintln!("Failed to exec: {}", error);
    std::process::exit(1);
}

pub fn handle_match(config: &NgrepConfig, args: Args, reader: Box<dyn BufRead>) -> Result<()> {
    let mut cli = Args::command();

    if args.pattern.is_none() {
        cli.error(
            ErrorKind::MissingRequiredArgument,
            "Missing required argument: pattern",
        )
        .exit();
    }

    let pattern_str = args.pattern.as_ref().unwrap();

    let model_path = config.resolve_model_path(
        &args
            .model
            .unwrap_or_else(|| config.default_model().unwrap()),
    )?;
    let matcher_threshold = args
        .threshold
        .unwrap_or_else(|| config.default_threshold().unwrap());
    let matcher_factory = EmbedNeuralMatcherFactory::new(model_path, matcher_threshold);

    let pattern = RegexBuilder::new(pattern_str)
        .neural_matcher_factory(&(Arc::new(matcher_factory) as Arc<dyn NeuralMatcherFactory>))
        .build()
        .context("Invalid regex pattern")?;

    let formatter = MatchFormatter::default();
    for (line_inx, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let captures = pattern
            .find_iter(line.as_str())
            .map(|cap| cap.unwrap())
            .collect::<Vec<_>>();

        if !captures.is_empty() {
            formatter.display(line_inx, &line, &captures);
        }
    }

    Ok(())
}
