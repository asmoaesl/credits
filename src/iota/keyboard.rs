use std::char;
use std::time::Duration;

// use rustbox::{RustBox, Event};
use crossterm::{InputEvent, KeyEvent, Crossterm};

// #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
// pub enum Key {
//     Tab,
//     Enter,
//     Esc,
//     Backspace,
//     Right,
//     Left,
//     Down,
//     Up,
//     Delete,
//     Home,
//     End,
//     CtrlLeft,
//     CtrlRight,

//     Char(char),
//     Ctrl(char),
// }

// impl Key {
//     pub fn from_special_code(code: u16) -> Option<KeyEvent> {
//         match code {
//             1     => Some(KeyEvent::Ctrl('a')),
//             2     => Some(KeyEvent::Ctrl('b')),
//             3     => Some(KeyEvent::Ctrl('c')),
//             4     => Some(KeyEvent::Ctrl('d')),
//             5     => Some(KeyEvent::Ctrl('e')),
//             6     => Some(KeyEvent::Ctrl('f')),
//             7     => Some(KeyEvent::Ctrl('g')),
//             8     => Some(KeyEvent::Ctrl('h')),
//             9     => Some(KeyEvent::Char('\t')),
//             13    => Some(KeyEvent::Char('\n')),
//             14    => Some(KeyEvent::Ctrl('n')),
//             16    => Some(KeyEvent::Ctrl('p')),
//             17    => Some(KeyEvent::Ctrl('q')),
//             18    => Some(KeyEvent::Ctrl('r')),
//             19    => Some(KeyEvent::Ctrl('s')),
//             24    => Some(KeyEvent::Ctrl('x')),
//             25    => Some(KeyEvent::Ctrl('y')),
//             26    => Some(KeyEvent::Ctrl('z')),
//             27    => Some(KeyEvent::Esc),
//             32    => Some(KeyEvent::Char(' ')),
//             127   => Some(KeyEvent::Backspace),
//             65514 => Some(KeyEvent::Right),
//             65515 => Some(KeyEvent::Left),
//             65516 => Some(KeyEvent::Down),
//             65517 => Some(KeyEvent::Up),
//             65520 => Some(KeyEvent::End),
//             65521 => Some(KeyEvent::Home),
//             65522 => Some(KeyEvent::Delete),
//             _     => None,
//         }
//     }

//     fn from_chord(rb: &mut Crossterm, start: u16) -> Option<Key> {
//         let chord = Self::get_chord(rb, start);

//         match chord.as_str() {
//             "\x1b[1;5C" => Some(Key::CtrlRight),
//             "\x1b[1;5D" => Some(Key::CtrlLeft),
//             _ => Key::from_special_code(start)
//         }
//     }

//     fn get_chord(rb: &mut Crossterm, start: u16) -> String {
//         // Copy any data waiting to a string
//         // There may be a cleaner way to do this?
//         let mut chord = char::from_u32(u32::from(start)).unwrap().to_string();
        
//         while let Some(InputEvent::Keyboard(KeyEvent::Char(ch))) = rb.input().read_sync().peekable().peek() {
//             chord.push(*ch);
//         }

//         chord
//     }
    
//     pub fn from_event(rb: &mut Crossterm, event: KeyEvent) -> Option<Key> {
//         match event {
//             // Event::KeyEventRaw(_, k, ch) => {
//             //     match k {
//             //         0 => char::from_u32(ch).map(Key::Char),
//             //         0x1b => Key::from_chord(rb, 0x1b),
//             //         a => Key::from_special_code(a)
//             //     }
//             // },
//             // TODO: this entire source file may need to be rewritten for crossterm
            
//             _ => None
//         }
//     }
// }
