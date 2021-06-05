use crate::layers::{Layer, MAX_LAYERS};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryFrom,
    fs::{self, File},
    io::{self, Write},
};
use thiserror::Error;

use anyhow::Context;

const DEFAULT_CONFIG: &str = include_str!("../config.ron");

pub type TitleMap = HashMap<String, Layer>;
pub fn get_map() -> Result<TitleMap, anyhow::Error> {
    let config = Config::from_file().context("failed to get config")?;

    config.into_map().context("Failed to parse config")
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct Config {
    // this is guaranteed to be 15 or less elements by the constructor
    layers: Vec<Vec<String>>,
}

impl Config {
    fn from_file() -> Result<Config, ConfigError> {
        let dirs = ProjectDirs::from(Default::default(), Default::default(), "qfo")
            .ok_or(ConfigError::NoHome)?;

        fs::create_dir_all(&dirs.config_dir())?;

        let config_file_path = dirs.config_dir().join("config.ron");

        use ron::de;
        let config: Config = if !config_file_path.exists() {
            // if our config doesn't exist we just write the default config into where it should be
            File::create(&config_file_path)?.write_all(DEFAULT_CONFIG.as_bytes())?;
            Config::default()
        } else {
            let config_file = File::open(&config_file_path)?;
            let config: Config = de::from_reader(config_file)?;

            if config.layers.len() >= MAX_LAYERS {
                return Err(ConfigError::TooManyLayers(config.layers.len()));
            } else {
                config
            }
        };

        Ok(config)
    }

    fn into_map(self) -> Result<HashMap<String, Layer>, ConfigError> {
        assert!(self.layers.len() < MAX_LAYERS);

        // iterate by value instead of by reference
        self.layers
            .into_iter()
            .enumerate()
            .map(|(i, data)| {
                (
                    // we are adding 1 because we are skipping the base layer
                    Layer::try_from(i + 1)
                        .expect("there is guaranteed to be one layer for each array item"),
                    data,
                )
            })
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
    }
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
