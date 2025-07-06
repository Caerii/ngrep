use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs::File, io::Write};

use anyhow::{Context, Result};
use expanduser::expanduser;
use toml_edit::DocumentMut;

const NGREP_HOME: &str = "~/.ngrep";
const NGREP_CONFIG: &str = "config.toml";
const NGREP_CONFIG_INIT: &str = r#"
# Ngrep configuration
#
# [model]
# default = <name>           # set a model as default
#
# [model.<name>]
# path = <ng path>           # path to .ng model relative to ~/.ngrep
# threshold = <number>       # default threshold for this model

[ngrep]
"#;

#[derive(Debug)]
pub struct NgrepConfig {
    path: PathBuf,
    conf: DocumentMut,
}

impl NgrepConfig {
    pub fn load_or_init() -> Result<NgrepConfig> {
        let path = expanduser(
            PathBuf::from_iter([NGREP_HOME, NGREP_CONFIG])
                .to_string_lossy()
                .to_string(),
        )?;

        if let Some(parent_dir) = path.parent() {
            fs::create_dir_all(parent_dir)?;
        }

        match File::create_new(&path) {
            Ok(mut handle) => handle.write_all(NGREP_CONFIG_INIT.as_bytes()),
            _ => Ok(()),
        }?;

        Self::load(&path)
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<NgrepConfig> {
        let mut config_file = File::open(&path)?;
        let mut config_toml: String = String::new();
        config_file.read_to_string(&mut config_toml)?;

        Ok(NgrepConfig {
            path: path.as_ref().into(),
            conf: config_toml
                .parse::<DocumentMut>()
                .context("Invalid toml configuration")?,
        })
    }

    pub fn home(&self) -> PathBuf {
        self.path
            .parent()
            .expect("Error getting parent")
            .to_path_buf()
    }

    pub fn path(&self) -> PathBuf {
        PathBuf::from_iter([self.home(), NGREP_CONFIG.into()])
    }

    pub fn default_model(&self) -> Result<String> {
        let model = self
            .conf
            .get("model")
            .context("Missing key 'model'")?
            .get("default")
            .context("Missing key 'model.default'")?
            .as_str()
            .context("Expected string type for key s'model.default'")?;

        Ok(model.to_string())
    }

    pub fn default_threshold(&self) -> Result<f32> {
        let def_model = self.default_model()?;
        let def_threshold = self
            .conf
            .get("model")
            .context("Missing key 'model'")?
            .get(&def_model)
            .context(format!("Missing key 'model.{}'", &def_model))?
            .get("threshold")
            .context(format!("Missing key 'model.{}.threshold'", &def_model))?
            .as_float()
            .context(format!(
                "Expected float type for key 'model.{}.threshold'",
                &def_model
            ))?;

        Ok(def_threshold as f32)
    }

    pub fn resolve_model_path(&self, name: &String) -> Result<PathBuf> {
        let model_path = &self
            .conf
            .get("model")
            .context("Missing key 'model'")?
            .get(name)
            .context(format!("Missing key 'model.{}'", name))?
            .get("path")
            .context(format!("Missing key 'model.{}.path'", name))?
            .as_str()
            .context(format!(
                "Expected string type for key 'model.{}.path'",
                name
            ))?;

        Ok(PathBuf::from_iter([self.home(), model_path.into()]))
    }

    pub fn add_model(&mut self, name: &str, path: &str) -> Result<()> {
        self.conf["model"] = toml_edit::table();
        self.conf["model"][name] = toml_edit::table();
        self.conf["model"][name]["path"] = toml_edit::value(path);

        self.sync()
    }

    fn sync(&self) -> Result<()> {
        let mut file = File::create(&self.path)?;

        file.write_all(self.conf.to_string().as_bytes())
            .context("Error updating configuration")
    }
}
