mod active_window_client;
mod config;
mod keyboard;
mod layers;

use config::{ClassRules, ConfigError};
use keyboard::{Keyboard, KeyboardError};
use layers::Layer;

use active_window_client::ActiveWindowClient;
use anyhow::Result;
use log::info;
use thiserror::Error;

fn main() -> Result<()> {
    // nice logging
    stderrlog::new()
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()?;

    let map: ClassRules = config::get_class_rules()?;

    let mut window_client = ActiveWindowClient::new()?;

    let mut keyboard = Keyboard::new()?;

    // this is to show that the last window we switched from was special
    let mut last_window_was_custom: bool = false;

    loop {
        window_client.wait_active_window(|class| {
            if let Some(layer) = map.layer(class) {
                info!("Setting layer to {:?}", layer);
                keyboard.set_layer(layer)?;
                last_window_was_custom = true;
            } else {
                // this is basically used to preserve manual layer changing at the hands of the
                // user. When a user focuses on a window that they have defined rules for
                // in the config, then we switch to that window's layer and it counts as a "custom"
                // switch, then if they focus on something else after that isn't "custom"
                // we don't blindly set their mapping to the base layer, as that would get
                // rid of any custom layers they have applied in the meantime.
                if last_window_was_custom {
                    info!("Resetting to base layer");
                    keyboard.set_layer(Layer::Zero)?;
                    last_window_was_custom = false;
                }
            }

            Ok(())
        })?;
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed on keyboard communication")]
    Keyboard {
        #[from]
        source: KeyboardError,
    },

    #[error(transparent)]
    Config(#[from] ConfigError),
}
