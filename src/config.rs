use crate::qmk::layers::{Layer, MAX_LAYERS};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryInto,
    fs::{self, File},
    io::{self, Write},
};
use thiserror::Error;

use anyhow::Context;

const DEFAULT_CONFIG: &str = include_str!("../config.ron");

type Id = u16;

#[derive(Debug, Clone)]
pub struct Config {
    pub rules: ClassRules,
    pub vendor_id: Id,
    pub product_id: Id,
}

#[derive(Debug, Clone)]
pub struct ClassRules(HashMap<String, Layer>);

impl ClassRules {
    pub fn with_raw_layers(
        layers: Vec<Vec<String>>,
        base_layer: Layer,
    ) -> Result<ClassRules, ConfigError> {
        // we still do this check because we want to provide meaningful error messages to the user
        if layers.len() >= MAX_LAYERS {
            return Err(ConfigError::TooManyLayers(layers.len()));
        }

        // iterates over each layer (skipping the base layer), and zipping this with the layers Vec, which guarantees that
        // our final result will be MAX_LAYERS - 1 or smaller in length
        Layer::iter()
            .skip(base_layer as usize + 1)
            .zip(layers.into_iter())
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
pub fn get_config() -> Result<Config, anyhow::Error> {
    let dirs = ProjectDirs::from("", "", "qfo").ok_or(ConfigError::NoHome)?;
    fs::create_dir_all(&dirs.config_dir())?;

    let RawConfig {
        raw_base_layer,
        raw_layers,
        product_id,
        vendor_id,
    } = get_config_from_file(dirs.config_dir().join("config.ron"))
        .context("failed to get config")?;

    // if we are given a base layer, we know that all layers are above or equal to it
    let base_layer = raw_base_layer
        .map(|layer| layer.try_into())
        .transpose()
        .context("failed while parsing base layer value")?
        .unwrap_or(Layer::Zero);

    let rules = ClassRules::with_raw_layers(raw_layers, base_layer)
        .context("failed while parsing rules")?;

    Ok(Config {
        rules,
        product_id,
        vendor_id,
    })
}

fn get_config_from_file(config_file_path: std::path::PathBuf) -> Result<RawConfig, ConfigError> {
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
    // product/vendor ids for your keyboard
    // NOTE: these are required because they are needed to find the keyboard among the usb devices,
    // and using some implicit table to find all QMK devices can lead to surprising effects.
    product_id: u16,
    vendor_id: u16,

    // default: Layer::Zero
    #[serde(rename = "base_layer")]
    raw_base_layer: Option<u8>,

    // these are layers provided by the config
    #[serde(rename = "layers")]
    raw_layers: Vec<Vec<String>>,
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
