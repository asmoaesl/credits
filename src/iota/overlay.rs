use std::cmp;
use std::convert::TryInto;

use unicode_width::UnicodeWidthStr;
// use rustbox::{Style, Color, RustBox};
use crossterm::{color, Attribute, Color, Colored, Colorize, Styler, Crossterm};

use editor::ALL_COMMANDS;
use command::BuilderEvent;
use keyboard::Key;
use keymap::CommandInfo;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OverlayType {
    CommandPrompt,
}

pub trait Overlay {
    fn draw(&self, rb: &mut Crossterm);
    fn draw_cursor(&mut self, rb: &mut Crossterm);
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent;
}

pub struct CommandPrompt {
    data: String,
    prefix: String,
    selected_index: usize,
}

impl CommandPrompt {
    pub fn new() -> CommandPrompt {
        CommandPrompt {
            data: String::new(),
            prefix: String::from(":"),
            selected_index: 0,
        }
    }
}

impl CommandPrompt {
    fn get_filtered_command_names(&self) -> Vec<&&str> {
        let mut keys: Vec<&&str> = ALL_COMMANDS
            .keys()
            .filter(|item| item.starts_with(&*self.data) )
            .collect();
        keys.sort();
        keys.reverse();

        keys
    }
}


impl Overlay for CommandPrompt {
    fn draw(&self, rb: &mut Crossterm) {
        let height = rb.terminal().terminal_size().1 - 1;
        let offset = self.prefix.len();

        let keys = self.get_filtered_command_names();

        // find the longest command in the resulting list
        let mut max = 20u16;
        for k in &keys {
            max = cmp::max(max, k.len().try_into().unwrap());
        }

        let cursor = rb.cursor();
        let terminal = rb.terminal();

        macro_rules! print_char {
            ($col:expr, $row:expr, $ch:expr) => {
                cursor.goto($col, $row).unwrap();
                terminal.write($ch).unwrap();
                terminal.write(Attribute::Reset).unwrap(); // Clear color and style settings
            };
        }

        // draw the command completion list
        let mut index = 1u16;
        for key in &keys {
            // rb.print_char(0, height - index, Style::empty(), Color::White, Color::Black, '│');
            // rb.print_char(max + 1, height - index, Style::empty(), Color::White, Color::Black, '│');
            print_char!(0, height - index, '│');
            print_char!(max + 1, height - index, '│');

            // let (fg, bg) = if index == self.selected_index.try_into().unwrap() {
            //     (Color::White, Color::Red)
            // } else {
            //     (Color::White, Color::Black)
            // };
            let selected = index as usize == self.selected_index; // If we're drawing the selected item

            let mut chars = key.chars();
            for x in 0..max {
                if let Some(ch) = chars.next() {
                    // rb.print_char(x + 1, height - index, Style::empty(), fg, bg, ch);
                    if selected {
                        print_char!(x + 1, height - index, format!("{}{}", Colored::Fg(Color::Red), ch));
                    } else {
                        print_char!(x + 1, height - index, ch);
                    }
                } else {
                    // rb.print_char(x + 1, height - index, Style::empty(), fg, bg, ' ');
                    if selected {
                        print_char!(x + 1, height - index, " ".red());
                    } else {
                        print_char!(x + 1, height - index, " ");
                    }
                }
            }

            index += 1;
        }

        // rb.print_char(0, height - index, Style::empty(), Color::White, Color::Black, '╭');
        print_char!(0, height - index, '╭');
        for x in 1..max + 1 {
            // rb.print_char(x, height - keys.len() - 1, Style::empty(), Color::White, Color::Black, '─');
            print_char!(x, height - keys.len() as u16 - 1, '─');
        }
        // rb.print_char(max + 1, height - index, Style::empty(), Color::White, Color::Black, '╮');
        print_char!(max + 1, height - index, '╮');

        // draw the given prefix
        for (index, ch) in self.prefix.chars().enumerate() {
            // rb.print_char(index, height, Style::empty(), Color::White, Color::Black, ch);
            print_char!(index.try_into().unwrap(), height, ch);
        }

        // draw the overlay data
        for (index, ch) in self.data.chars().enumerate() {
            // rb.print_char(index + offset, height, Style::empty(), Color::White, Color::Black, ch);
            print_char!((index + offset) as u16, height, ch);
        }
    }

    fn draw_cursor(&mut self, rb: &mut Crossterm) {
        // Prompt is always on the bottom, so we can use the
        // height given by the frontend here
        let height = rb.terminal().terminal_size().1 - 1;
        let prefix_len = UnicodeWidthStr::width(self.prefix.as_str());
        let data_len = UnicodeWidthStr::width(self.data.as_str());
        let cursor_x = prefix_len + data_len;
        rb.cursor().goto(cursor_x as u16, height);
    }

    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        match key {
            Key::Esc => {
                let command_info = CommandInfo {
                    command_name: String::from("editor::noop"),
                    args: None,
                };
                return BuilderEvent::Complete(command_info);
            }
            Key::Backspace => { self.data.pop(); },
            Key::Enter => {
                let command_info = CommandInfo {
                    command_name: self.data.clone(),
                    args: None,
                };
                return BuilderEvent::Complete(command_info);
            }
            Key::Up => {
                let max = self.get_filtered_command_names().len();
                if self.selected_index < max {
                    self.selected_index += 1;
                }
            }
            Key::Down => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            Key::Tab => {
                if self.selected_index > 0 {
                    let command = {
                        let mut keys: Vec<&&str> = ALL_COMMANDS
                            .keys()
                            .filter(|item| item.starts_with(&*self.data) )
                            .collect();
                        keys.sort();
                        keys.reverse();

                        keys[self.selected_index - 1].clone()
                    };
                    self.data = command.to_string();
                }
            }
            Key::Char(c) => { self.data.push(c) },
            _ => {}
        }
        BuilderEvent::Incomplete
    }
}
