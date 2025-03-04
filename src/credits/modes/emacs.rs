use crossterm::KeyEvent;
use crate::keymap::{KeyMap, KeyMapState, CommandInfo};
use crate::command::{BuilderEvent, BuilderArgs };
use crate::textobject::{Offset, Anchor, Kind};
use crate::buffer::Mark;

use super::Mode;

/// Emacs mode uses Emacs-like keybindings.
///
pub struct EmacsMode {
    keymap: KeyMap,
    match_in_progress: bool,
}

impl EmacsMode {

    /// Create a new instance of EmacsMode
    pub fn new() -> EmacsMode {
        EmacsMode {
            keymap: EmacsMode::key_defaults(),
            match_in_progress: false,
        }
    }

    /// Creates a KeyMap with default EmacsMode key bindings
    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();

        // Editor Commands
        keymap.bind_keys(
            &[KeyEvent::Ctrl('x'), KeyEvent::Ctrl('c')],
            CommandInfo {
                command_name: String::from("editor::quit"),
                args: None,
            }
        );
        keymap.bind_keys(
            &[KeyEvent::Ctrl('x'), KeyEvent::Ctrl('s')],
            CommandInfo {
                command_name: String::from("editor::save_buffer"),
                args: None,
            }
        );

        // Cursor movement
        keymap.bind_key(
            KeyEvent::Up,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::Same))
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Down,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::Same))
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Left,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Right,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Ctrl('p'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::Same))
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Ctrl('n'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::Same))
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Ctrl('b'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Ctrl('f'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
            }
        );

        keymap.bind_key(
            KeyEvent::Ctrl('e'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::End))
                                             .with_offset(Offset::Forward(0, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Ctrl('a'),
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::End))
                                             .with_offset(Offset::Backward(0, Mark::Cursor(0))))
            }
        );

        // Editing
        keymap.bind_key(
            KeyEvent::Char('\t'),
            CommandInfo {
                command_name: String::from("buffer::insert_tab"),
                args: None,
            }
        );
        keymap.bind_key(
            KeyEvent::Char('\n'),
            CommandInfo {
                command_name: String::from("buffer::insert_char"),
                args: Some(BuilderArgs::new().with_char_arg('\n')),
            }
        );
        keymap.bind_key(
            KeyEvent::Backspace,
            CommandInfo {
                command_name: String::from("buffer::delete_char"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Delete,
            CommandInfo {
                command_name: String::from("buffer::delete_char"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Ctrl('h'),
            CommandInfo {
                command_name: String::from("buffer::delete_char"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            KeyEvent::Ctrl('d'),
            CommandInfo {
                command_name: String::from("buffer::delete_char"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_keys(
            &[KeyEvent::Ctrl('x'), KeyEvent::Ctrl('f')],
            CommandInfo {
                command_name: String::from("editor::find_file"),
                args: None,
            }
        );
        // keymap.bind_keys(
        //     &[Key::Ctrl('x'), Key::Ctrl('f')],
        //     CommandInfo {
        //         command_name: String::from("editor::switch_to_last_buffer"),
        //         args: None,
        //     }
        // );

        keymap
    }

    /// Checks a Key against the internal keymap
    ///
    /// - If there is a direct match, return the completed BuilderEvent
    /// - If there is a partial match, set match_in_progress to true which
    ///   indicates that the next key should check against the keymap too,
    ///   rather than possibly being inserted into the buffer. This allows
    ///   for non-prefixed keys to be used in keybindings. ie: C-x s rather
    ///   than C-x C-s.
    /// - If there is no match of any kind, return Incomplete
    fn check_key(&mut self, key: KeyEvent) -> BuilderEvent {
        match self.keymap.check_key(key) {
            KeyMapState::Match(c) => {
                self.match_in_progress = false;
                BuilderEvent::Complete(c)
            },
            KeyMapState::Continue => {
                self.match_in_progress = true;
                BuilderEvent::Incomplete
            }
            KeyMapState::None => {
                self.match_in_progress = false;
                BuilderEvent::Incomplete
            }
        }
    }

}

impl Mode for EmacsMode {
    /// Given a key, pass it through the EmacsMode KeyMap and return the associated Command, if any.
    /// If no match is found, treat it as an InsertChar command.
    fn handle_key_event(&mut self, key: KeyEvent) -> BuilderEvent {
        if self.match_in_progress {
            return self.check_key(key)
        }

        if let KeyEvent::Char(c) = key {
            let mut builder_args = BuilderArgs::new().with_char_arg(c);
            let command_info = CommandInfo {
                command_name: String::from("buffer::insert_char"),
                args: Some(builder_args),
            };
            BuilderEvent::Complete(command_info)
        } else {
            self.check_key(key)
        }

    }
}

impl Default for EmacsMode {
    fn default() -> Self {
        Self::new()
    }
}
