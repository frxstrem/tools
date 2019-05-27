use crate::message::Message;

pub trait MessagePrinter {
    fn print(&self, message: &Message);
}

pub struct PlainPrinter;

impl MessagePrinter for PlainPrinter {
    fn print(&self, message: &Message) {
        println!("{}", message);
    }
}
