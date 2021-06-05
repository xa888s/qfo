use crate::layers::Layer;
use hidapi::{HidApi, HidDevice, HidError};
use thiserror::Error;

// TODO: not hard code it to just my keyboard
const VENDOR_ID: u16 = 0x3297;
const PRODUCT_ID: u16 = 0x4974;

pub type KeyboardResult<T> = Result<T, KeyboardError>;

#[derive(Debug, Error)]
pub enum KeyboardError {
    #[error("error in communication with keyboard")]
    HidError(#[from] HidError),

    // FIXME: don't expose this to the user
    #[error("invalid length/data {0}")]
    InvalidLayer(u8),

    #[error("keyboard not found")]
    NotFound,
}

pub struct Keyboard {
    device: HidDevice,
}

impl Keyboard {
    pub fn new() -> KeyboardResult<Keyboard> {
        let api = HidApi::new()?;

        // takes the 2nd HidDevice because the first one doesn't accept messages/data
        let device = api
            .device_list()
            .filter(|d| d.vendor_id() == VENDOR_ID && d.product_id() == PRODUCT_ID)
            .nth(1)
            .ok_or(KeyboardError::NotFound)?
            .open_device(&api)?;

        Ok(Keyboard { device })
    }

    pub fn set_layer(&mut self, layer: Layer) -> Result<(), KeyboardError> {
        let send = [0, layer as u8];

        let sent = self.device.write(&send)?;
        assert_eq!(send.len(), sent);

        let mut recv = [0; 1];
        let read = self.device.read(&mut recv[..])?;
        assert_eq!(recv.len(), read);

        Keyboard::code_to_result(recv[0])
    }

    fn code_to_result(code: u8) -> KeyboardResult<()> {
        // TODO: define all error kinds
        match code {
            0 => Ok(()),
            1 => Err(KeyboardError::InvalidLayer(code)),
            _ => unreachable!(),
        }
    }
}
