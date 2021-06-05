mod config;
mod keyboard;
mod layers;

use config::{ConfigError, TitleMap};
use keyboard::{Keyboard, KeyboardError};
use layers::Layer;

use anyhow::{Context, Result};
use std::{borrow::Cow, convert::TryInto};
use thiserror::Error;

use x11rb::{
    connection::Connection,
    properties::WmClass,
    protocol::xproto::{
        AtomEnum, ChangeWindowAttributesAux, ChangeWindowAttributesRequest, ConnectionExt,
        EventMask, Window,
    },
    rust_connection::RustConnection,
};

fn main() -> Result<()> {
    let map: TitleMap = config::get_map()?;

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
