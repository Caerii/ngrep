mod commands;
mod config;
mod formatters;
mod neural_matchers;

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Error, Result};
use clap::{Parser, Subcommand};
use commands::{handle_config, handle_import, handle_match};
use config::NgrepConfig;

// --- Command Line Arguments
#[derive(Parser, Debug)]
#[command(name = "ngrep")]
#[command(version = "0.1.0")]
#[command(version, about, about = "A neural grep")]
pub struct Args {
    #[command(subcommand, name = "command")]
    command: Option<Commands>,

    /// Model name
    #[arg(short, long)]
    model: Option<String>,

    /// Similarity threshold
    #[arg(short, long)]
    threshold: Option<f32>,

    /// The search pattern
    #[arg(name = "pattern")]
    pattern: Option<String>,

    /// File to search in or '-' for stdin.
    #[arg(name = "file", default_value = "-")]
    file: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Import a supported model converting it to ng format
    /// and making it the default one
    Import {
        /// Path to a model
        #[arg(short, long)]
        path: PathBuf,

        /// Name of the model
        #[arg(short, long)]
        name: String,
    },
    /// Edit ~/.ngrep/config.toml
    Config,
}

fn reader(path: &str) -> Result<Box<dyn BufRead>> {
    match path {
        "-" => Ok(Box::new(io::stdin().lock())),
        _ => {
            let file =
                File::open(&Path::new(path)).context(format!("failed to open: '{}'", path))?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}

fn main() -> Result<(), Error> {
    let mut config = NgrepConfig::load_or_init()?;
    let args: Args = Args::parse();

    if let Some(command) = args.command {
        return match command {
            Commands::Import { path, name } => handle_import(&mut config, path, &name),
            Commands::Config {} => handle_config(&config),
        };
    }

    let input = reader(&args.file.clone())?;
    handle_match(&config, args, input)
}
