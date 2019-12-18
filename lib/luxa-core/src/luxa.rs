use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::*;

#[async_trait(?Send)]
pub trait Luxafor {
    async fn solid(&self, color: Color) -> Result<(), LuxaError>;
    async fn fade(&self, color: Color, duration: u8) -> Result<(), LuxaError>;
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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
