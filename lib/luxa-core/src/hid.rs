use crate::error::*;
use crate::luxa::*;

use hidapi::{HidApi, HidDevice};

pub const LUXAFOR_VENDOR_ID: u16 = 0x04d8;
pub const LUXAFOR_PRODUCT_ID: u16 = 0xf372;

pub struct LuxaforHid {
    hid_device: HidDevice,
}

impl LuxaforHid {
    pub fn open(
        hid_api: &HidApi,
        vendor_id: u16,
        product_id: u16,
    ) -> Result<LuxaforHid, LuxaError> {
        let hid_device = hid_api.open(vendor_id, product_id)?;
        Ok(LuxaforHid { hid_device })
    }

    pub fn open_default() -> Result<LuxaforHid, LuxaError> {
        let hid_api = HidApi::new()?;
        Self::open(&hid_api, LUXAFOR_VENDOR_ID, LUXAFOR_PRODUCT_ID)
    }

    fn write(&self, data: &[u8]) -> Result<(), LuxaError> {
        self.hid_device.write(data)?;
        Ok(())
    }
}

impl Luxafor for LuxaforHid {
    fn solid(&self, color: Color) -> Result<(), LuxaError> {
        let (r, g, b) = color.to_rgb();
        self.write(&[1, Leds::All as u8, r, g, b])
    }

    fn fade(&self, color: Color, duration: u8) -> Result<(), LuxaError> {
        let (r, g, b) = color.to_rgb();
        self.write(&[2, Leds::All as u8, r, g, b, duration])
    }
}
