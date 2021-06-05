mod keyboard;
mod layers;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use keyboard::{Keyboard, KeyboardError};
use layers::{Layer, MAX_LAYERS};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
    convert::{TryFrom, TryInto},
    fs::{self, File},
    io::{self, Write},
};
use thiserror::Error;

use x11rb::{
    connection::Connection,
    properties::WmClass,
    protocol::xproto::{
        self, AtomEnum, ChangeWindowAttributesAux, ChangeWindowAttributesRequest, ConnectionExt,
        EventMask, Window,
    },
    rust_connection::RustConnection,
};

const DEFAULT_CONFIG: &str = include_str!("../config.ron");

fn main() -> Result<()> {
    let map: TitleMap = get_map()?;

    let (conn, screen_num) =
        RustConnection::connect(None).context("failed to connect to X11 server")?;

    let active_window = conn.intern_atom(true, b"_NET_ACTIVE_WINDOW")?.reply()?;

    let screen = &conn.setup().roots[screen_num];

    let aux = ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE);

    ChangeWindowAttributesRequest {
        window: screen.root,
        value_list: Cow::Owned(aux),
    }
    .send(&conn)?;

    conn.flush()?;

    let mut keyboard = Keyboard::new()?;

    loop {
        let event = conn.wait_for_event().context("failed to get event")?;

        use x11rb::protocol::Event;
        if let Event::PropertyNotify(e) = event {
            if e.atom == active_window.atom {
                let active_window = conn
                    .get_property(
                        false,
                        screen.root,
                        active_window.atom,
                        AtomEnum::WINDOW,
                        0,
                        4,
                    )
                    .context("failed to get active window")?
                    .reply()
                    .context("failed to get active window reply")?;

                let bytes: [u8; 4] = active_window.value[..]
                    .try_into()
                    .context("failed to parse active window into u32")?;
                let window = Window::from_ne_bytes(bytes);

                let class = match WmClass::get(&conn, window)
                    .context("failed to get WM_CLASS")?
                    .reply()
                {
                    Ok(class) => class,
                    // TODO: check for a specific error, instead of just ignoring all orders
                    Err(_) => continue,
                };

                let class =
                    std::str::from_utf8(class.class()).context("failed to parse class as utf-8")?;

                if let Some(&layer) = map.get(class) {
                    keyboard.set_layer(layer)?;
                } else {
                    // default layer
                    keyboard.set_layer(Layer::Zero)?;
                }
            }
        }
    }

    // keyboard.set_layer(Layer::One)?;
}

type TitleMap = HashMap<String, Layer>;
fn get_map() -> Result<TitleMap, anyhow::Error> {
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
enum ConfigError {
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

#[derive(Debug, Error)]
enum Error {
    #[error("failed on keyboard communication")]
    Keyboard {
        #[from]
        source: KeyboardError,
    },

    #[error(transparent)]
    Config(#[from] ConfigError),
}
