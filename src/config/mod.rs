use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RouteConfig {
    pub general: Option<General>,
    pub input: Input,
    pub output: Output,
}

#[derive(Deserialize, Debug)]
pub struct General {
    pub private_middleware_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Output {
    pub path: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct Input {
    pub module_path: PathBuf,
}

pub fn config_path() -> Result<PathBuf, errors::ConfigError> {
    let mut cwd = current_dir()?;

    loop {
        let candidate = cwd.join("route.toml");

        if candidate.exists() {
            return Ok(candidate);
        }
        if !cwd.pop() {
            return Err(errors::ConfigError::no_config());
        }
    }
}

fn resolve_path(path: &Path, root: &Path) -> PathBuf {
    if path.is_relative() {
        root.join(&path)
    } else {
        path.to_path_buf()
    }
}

impl RouteConfig {
    pub fn read() -> Result<Self, errors::ConfigError> {
        let config_file = config_path()?;
        let root = config_file.parent().unwrap();

        let mut config: RouteConfig = toml::from_slice(&fs::read(&config_file)?)?;

        config.input.module_path = resolve_path(&config.input.module_path, root);
        config.output.path = resolve_path(&config.output.path, root);

        Ok(config)
    }
}

pub mod errors {
    use thiserror::Error;
    use thiserror_ext::{Box, Construct};

    #[derive(Error, Debug, Box, Construct)]
    #[thiserror_ext(newtype(name = ConfigError))]
    pub enum ConfigErrorKind {
        #[error("io error")]
        Io(#[from] std::io::Error),
        #[error("unable to find route.toml config file")]
        NoConfig,

        #[error("error deserializing configuration file")]
        De(#[from] toml::de::Error),
    }
}
