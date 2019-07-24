use crate::error::*;
use crate::luxa::*;

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    Red,
    Blue,
    Green,
    Cyan,
    Magenta,
    Yellow,
    White,
    Black,
}

pub const ALL_MODES: &[Mode] = &[
    Mode::Red,
    Mode::Blue,
    Mode::Green,
    Mode::Cyan,
    Mode::Magenta,
    Mode::Yellow,
    Mode::White,
    Mode::Black,
];

pub fn get_all_modes() -> &'static [Mode] {
    return ALL_MODES;
}

impl Mode {
    pub fn names(&self) -> &'static [&'static str] {
        match self {
            Mode::Red => &["red", "busy"],
            Mode::Blue => &["blue"],
            Mode::Green => &["green", "available", "ok"],
            Mode::Cyan => &["cyan"],
            Mode::Magenta => &["magenta"],
            Mode::Yellow => &["yellow"],
            Mode::White => &["white"],
            Mode::Black => &["black", "none", "blank"],
        }
    }

    pub fn run(&self, device: &Luxafor) -> Result<(), LuxaError> {
        match self {
            Mode::Red => device.fade(Color::RED, 50),
            Mode::Blue => device.fade(Color::BLUE, 50),
            Mode::Green => device.fade(Color::GREEN, 50),
            Mode::Cyan => device.fade(Color::CYAN, 50),
            Mode::Magenta => device.fade(Color::MAGENTA, 50),
            Mode::Yellow => device.fade(Color::YELLOW, 50),
            Mode::White => device.fade(Color::WHITE, 50),
            Mode::Black => device.fade(Color::BLACK, 50),
        }
    }
}
