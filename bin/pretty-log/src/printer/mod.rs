use crate::message::Message;

pub mod styling;
pub use styling::*;

pub mod plain;
pub use plain::*;

pub mod fancy;
pub use fancy::*;

pub trait MessagePrinter: Send + Sync {
    fn print(&self, message: &Message);

    fn emphasize(&self, text: &str) -> String;
}
