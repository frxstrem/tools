use crate::error::*;

use hidapi::{HidApi, HidDevice};

pub const LUXAFOR_VENDOR_ID: u16 = 0x04d8;
pub const LUXAFOR_PRODUCT_ID: u16 = 0xf372;

pub struct Luxafor {
    hid_device: HidDevice,
}

impl Luxafor {
    pub fn open(hid_api: &HidApi, vendor_id: u16, product_id: u16) -> Result<Luxafor, LuxaError> {
        let hid_device = hid_api.open(vendor_id, product_id)?;
        Ok(Luxafor { hid_device })
    }

    pub fn open_default() -> Result<Luxafor, LuxaError> {
        let hid_api = HidApi::new()?;
        Self::open(&hid_api, LUXAFOR_VENDOR_ID, LUXAFOR_PRODUCT_ID)
    }

    fn write(&self, data: &[u8]) -> Result<(), LuxaError> {
        self.hid_device.write(data)?;
        Ok(())
    }

    pub fn solid(&self, color: Color) -> Result<(), LuxaError> {
        let (r, g, b) = color.to_rgb();
        self.write(&[1, Leds::All as u8, r, g, b])
    }

    pub fn fade(&self, color: Color, duration: u8) -> Result<(), LuxaError> {
        let (r, g, b) = color.to_rgb();
        self.write(&[2, Leds::All as u8, r, g, b, duration])
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Leds {
    All = 0xff,
}

#[derive(Debug, Copy, Clone)]
pub enum Color {
    Rgb(u8, u8, u8),
}

impl Color {
    pub const RED: Color = Color::Rgb(64, 0, 0);
    pub const GREEN: Color = Color::Rgb(0, 64, 0);
    pub const BLUE: Color = Color::Rgb(0, 0, 64);
    pub const CYAN: Color = Color::Rgb(0, 64, 64);
    pub const MAGENTA: Color = Color::Rgb(64, 0, 64);
    pub const YELLOW: Color = Color::Rgb(64, 64, 0);
    pub const WHITE: Color = Color::Rgb(64, 64, 64);
    pub const BLACK: Color = Color::Rgb(0, 0, 0);

    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::Rgb(r, g, b) => (*r, *g, *b),
        }
    }
}
