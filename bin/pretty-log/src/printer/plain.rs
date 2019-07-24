use std::marker::PhantomData;

use super::styling::*;
use super::MessagePrinter;
use crate::message::Message;

#[derive(Copy, Clone)]
pub struct PlainPrinter<S: Styling>(PhantomData<S>);

impl<S: Styling> Default for PlainPrinter<S> {
    fn default() -> PlainPrinter<S> {
        PlainPrinter(PhantomData)
    }
}

impl<S: Styling> MessagePrinter for PlainPrinter<S> {
    fn print(&self, message: &Message) {
        println!(
            "{}{}{}",
            S::severity(message.severity()),
            message,
            S::reset()
        );
    }

    fn emphasize(&self, text: &str) -> String {
        format!("{}{}{}", S::emphasize(), text, S::no_emphasize())
    }
}
