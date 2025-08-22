use clap::error::ErrorKind;
use clap::CommandFactory;
use edit::get_editor;
use fancy_regex::{NeuralMatcherFactory, RegexBuilder};
use std::io::BufRead;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::config::{ModelConfig, NgrepConfig};
use crate::formatters::{MatchFormatter, MatchFormatterOptions};
use crate::neural_matchers::EmbedNeuralMatcherFactory;
use crate::Args;
use embeddings::converters::{self, Formats};
use embeddings::ng;

pub fn handle_import<P: AsRef<Path>>(
    config: &mut NgrepConfig,
    path: P,
    name: &str,
    threshold: f64,
    set_default: bool,
) -> Result<()> {
    let model_file_name = path
        .as_ref()
        .file_name()
        .context("Error getting model file name")?;
    let model_path = config
        .home()
        .join(model_file_name)
        .with_extension(ng::NG_EXTENSION);
    let model_conf = ModelConfig::new(name.into(), model_path, threshold)?;

    converters::to_ng(Formats::Text, path.as_ref(), &model_conf.path)
        .context("Error during import of the model")?;

    config.add_model(&model_conf, set_default)
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

    // --- model initialization
    let model_config = config
        .model()
        .context("No default model found, run `ngrep import` first")?;
    let neural_matcher = EmbedNeuralMatcherFactory::new(&model_config.path, model_config.threshold)
        .context("Error during model initialization")?;
    let neural_regex = RegexBuilder::new(&args.pattern.unwrap())
        .neural_matcher_factory(&(Arc::new(neural_matcher) as Arc<dyn NeuralMatcherFactory>))
        .build()
        .context("Invalid regex pattern")?;

    // --- matching loop
    let formatter = MatchFormatter::new(
        MatchFormatterOptions::default()
            .with_line_number(args.line_number)
            .with_only_matching(args.only_matching),
    );

    let mut stdout = std::io::stdout().lock();
    for (inx, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let captures: Vec<(usize, usize)> = neural_regex
            .find_iter(line.as_str())
            .map(|cap| cap.unwrap())
            .map(|cap| (cap.start(), cap.end()))
            .collect();

        if !captures.is_empty() {
            formatter.display_line(&mut stdout, inx, &line, &captures);
        }
    }

    Ok(())
}
