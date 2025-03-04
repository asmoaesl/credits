use crossterm::KeyEvent;
use crate::keymap::{KeyMap, KeyMapState, CommandInfo};
use crate::command::{BuilderEvent, BuilderArgs };
use crate::textobject::{ Offset, Kind, Anchor };
use crate::buffer::Mark;
use crate::overlay::OverlayType;
use crate::modes::ModeType;

use super::Mode;


/// `NormalMode` mimics Vi's Normal mode.
pub struct NormalMode {
    keymap: KeyMap,
    number: Option<i32>,
    reading_number: bool,
}

impl NormalMode {

    /// Create a new instance of `NormalMode`
    pub fn new() -> NormalMode {
        NormalMode {
            keymap: NormalMode::key_defaults(),
            number: None,
            reading_number: false,
        }
    }

    /// Creates a `KeyMap` with default `NormalMode` key bindings
    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();
        // movement
        // { keys: 'h', command: 'buffer::move_cursor', args: { direction: backward, kind: char, number: 1 } }
        keymap.bind_key(
            KeyEvent::Char('h'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Char('j'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::Same))
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Char('k'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::Same))
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Char('l'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Char('w'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Word(Anchor::Start))
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Char('b'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Word(Anchor::Start))
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Char('$'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::End))
                                             .with_offset(Offset::Forward(0, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Char('0'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::Start))
                                             .with_offset(Offset::Backward(0, Mark::Cursor(0))))
            }
        );

        // actions
        keymap.bind_key(
            KeyEvent::Char('u'),
            CommandInfo {
                command_name: String::from("editor::undo"),
                args: None,
            }
        );
        keymap.bind_key(
            KeyEvent::Ctrl('r'),
            CommandInfo {
                command_name: String::from("editor::redo"),
                args: None,
            }
        );

        keymap.bind_key(
            KeyEvent::Char('i'),
            CommandInfo {
                command_name: String::from("editor::set_mode"),
                args: Some(BuilderArgs::new().with_mode(ModeType::Insert)),
            }
        );
        keymap.bind_key(
            KeyEvent::Char(':'),
            CommandInfo {
                command_name: String::from("editor::set_overlay"),
                args: Some(BuilderArgs::new().with_overlay(OverlayType::CommandPrompt)),
            }
        );

        keymap
    }

}

impl Mode for NormalMode {
    fn handle_key_event(&mut self, key: KeyEvent) -> BuilderEvent {
        if let KeyEvent::Char(c) = key {
            // '0' might be bound (start of line), and cannot be the start of a number sequence
            if c.is_digit(10) && (self.reading_number || c != '0') {
                let n = c.to_digit(10).unwrap() as i32;
                self.reading_number = true;
                if let Some(current) = self.number {
                    self.number = Some((current*10) + n);
                } else {
                    self.number = Some(n);
                }
                return BuilderEvent::Incomplete;
            } else if self.reading_number {
                self.reading_number = false;
            }
        }
        match self.keymap.check_key(key) {
            KeyMapState::Match(mut c) => {
                if let Some(num) = self.number {
                    if let Some(args) = c.args {
                        c.args = Some(args.with_number(num));
                    }
                }
                self.number = None;
                BuilderEvent::Complete(c)
            }
            _ => {
                BuilderEvent::Incomplete
            }
        }
    }
}

impl Default for NormalMode {
    fn default() -> Self {
        Self::new()
    }
}
