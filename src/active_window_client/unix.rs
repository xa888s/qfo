use std::{borrow::Cow, convert::TryInto};
use thiserror::Error;

use x11rb::{
    connection::Connection,
    properties::WmClass,
    protocol::xproto::{
        Atom, AtomEnum, ChangeWindowAttributesAux, ChangeWindowAttributesRequest, ConnectionExt,
        EventMask, Window,
    },
    rust_connection::{ConnectError, RustConnection},
};

pub struct ActiveWindowClient {
    conn: RustConnection,
    active_window_atom: Atom,
    screen_num: usize,
}

#[derive(Debug, Error)]
pub enum ActiveWindowError {
    #[error(transparent)]
    InitialConnection(#[from] ConnectError),

    #[error(transparent)]
    DuringConnection(#[from] x11rb::errors::ConnectionError),

    #[error(transparent)]
    Reply(#[from] x11rb::errors::ReplyError),

    #[error(transparent)]
    Callback(#[from] crate::Error),
}

impl ActiveWindowClient {
    pub fn new() -> Result<ActiveWindowClient, ActiveWindowError> {
        let (conn, screen_num) = RustConnection::connect(None)?;

        let active_window = conn.intern_atom(true, b"_NET_ACTIVE_WINDOW")?.reply()?;

        let screen = &conn.setup().roots[screen_num];

        let aux = ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE);

        ChangeWindowAttributesRequest {
            window: screen.root,
            value_list: Cow::Owned(aux),
        }
        .send(&conn)?;

        conn.flush()?;

        Ok(ActiveWindowClient {
            conn,
            active_window_atom: active_window.atom,
            screen_num,
        })
    }

    // doesn't guarantee that it will run the callback, it will only run it if there is an active
    // window returned from the event
    pub fn wait_active_window(
        &mut self,
        mut cb: impl FnMut(&str) -> Result<(), crate::Error>,
    ) -> Result<(), ActiveWindowError> {
        let event = self.conn.wait_for_event()?;

        use x11rb::protocol::Event;
        if let Event::PropertyNotify(e) = event {
            if e.atom == self.active_window_atom {
                let active_window = self
                    .conn
                    .get_property(
                        false,
                        self.conn.setup().roots[self.screen_num].root,
                        self.active_window_atom,
                        AtomEnum::WINDOW,
                        0,
                        4,
                    )?
                    .reply()?;

                let bytes: [u8; 4] = active_window.value[..]
                    .try_into()
                    .expect("The server should always return a valid slice");

                let window = Window::from_ne_bytes(bytes);

                let class = match WmClass::get(&self.conn, window)?.reply() {
                    Ok(class) => class,
                    // TODO: check for a specific error, instead of just ignoring all orders
                    Err(_) => return Ok(()),
                };

                // only call if there actually is a string
                if let Ok(class) = std::str::from_utf8(class.class()) {
                    cb(class)?;
                }
            }
        }
        Ok(())
    }
}
