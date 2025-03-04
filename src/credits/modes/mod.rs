use crossterm::KeyEvent;
use crate::command::BuilderEvent;

pub use self::standard::StandardMode;
pub use self::normal::NormalMode;
pub use self::insert::InsertMode;
pub use self::emacs::EmacsMode;

mod standard;
mod normal;
mod insert;
mod emacs;

#[derive(Copy, Clone, Debug)]
pub enum ModeType {
    Normal,
    Insert,
}

/// The concept of Iota's modes are taken from Vi.
/// 
/// A mode is a mechanism for interpreting key events and converting them into
/// commands which the Editor will interpret.
pub trait Mode {
    /// Given a Key, return a Command wrapped in a BuilderEvent for the Editor to interpret
    fn handle_key_event(&mut self, key: KeyEvent) -> BuilderEvent;
}
