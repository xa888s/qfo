use crate::layers::{Layer, MAX_LAYERS};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    fs::{self, File},
    io::{self, Write},
};
use thiserror::Error;

use anyhow::Context;

const DEFAULT_CONFIG: &str = include_str!("../config.ron");

#[derive(Debug, Clone)]
pub struct ClassRules(HashMap<String, Layer>);

impl ClassRules {
    pub fn with_layers(layers: Vec<Vec<String>>) -> Result<ClassRules, ConfigError> {
        // we still do this check because we want to provide meaningful error messages to the user
        if layers.len() >= MAX_LAYERS {
            return Err(ConfigError::TooManyLayers(layers.len()));
        }

        // iterates over each layer (skipping the first (base layer), and zipping this with the layers Vec, which guarantees that
        // our final result will be MAX_LAYERS - 1 or smaller in length
        Layer::iter()
            .skip(1)
            .zip(layers.into_iter())
            .map(|(layer, data)| (layer, data))
            .try_fold(HashMap::new(), |mut map, (layer, data)| {
                for (i, title) in data.into_iter().enumerate() {
                    // if there is something already there, we return early with an error and
                    // some info. If there is nothing we just insert like normal.
                    match map.entry(title) {
                        Entry::Occupied(occupied) => {
                            let (title, _) = occupied.remove_entry();
                            return Err(ConfigError::DuplicateTitle {
                                title,
                                layer,
                                title_index: i,
                            });
                        }
                        Entry::Vacant(vacancy) => {
                            vacancy.insert(layer);
                        }
                    }
                }
                Ok(map)
            })
            .map(ClassRules)
    }

    pub fn layer(&self, class: &str) -> Option<Layer> {
        self.0.get(class).copied()
    }
}

// helper function to guarantee a safe/correct interface for main
pub fn get_class_rules() -> Result<ClassRules, anyhow::Error> {
    let dirs = ProjectDirs::from(Default::default(), Default::default(), "qfo")
        .ok_or(ConfigError::NoHome)?;
    fs::create_dir_all(&dirs.config_dir())?;

    let config = from_file(dirs.config_dir().join("config.ron")).context("failed to get config")?;

    ClassRules::with_layers(config.layers).context("failed while parsing rules")
}

fn from_file(config_file_path: std::path::PathBuf) -> Result<RawConfig, ConfigError> {
    use ron::de;
    let config: RawConfig = if !config_file_path.exists() {
        // if our config doesn't exist we just write the default config into where it should be
        File::create(&config_file_path)?.write_all(DEFAULT_CONFIG.as_bytes())?;
        RawConfig::default()
    } else {
        let config_file = File::open(&config_file_path)?;
        let config: RawConfig = de::from_reader(config_file)?;

        config
    };

    Ok(config)
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct RawConfig {
    // these are layers provided by the config
    pub layers: Vec<Vec<String>>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    Create(#[from] io::Error),

    #[error("failed to parse config file")]
    Parse(#[from] ron::Error),

    #[error(
        "your config file has too many layers (max: {}, actual: {0})",
        MAX_LAYERS - 1
    )]
    TooManyLayers(usize),

    #[error(
        "the title {title} is duplicated in the {layer:?}-th layer at the {title_index}th title"
    )]
    DuplicateTitle {
        title: String,
        layer: Layer,
        title_index: usize,
    },

    #[error("failed to get project directories")]
    NoHome,
}
