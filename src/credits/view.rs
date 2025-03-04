// For a list of 256 terminal colors: https://jonasjacek.github.io/colors/

use std::cmp;
use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::io::Write;
use std::fs::{File, rename};
use std::sync::{Mutex, Arc};
use std::time::SystemTime;

// use rustbox::{Color, RustBox, Style as RustBoxStyle};
use crossterm::{Color, TerminalCursor, Colorize, Colored, Styler, Attribute, Crossterm};

use tempdir::TempDir;
use unicode_width::UnicodeWidthChar;

use crate::buffer::{Buffer, Mark};
use crate::overlay::{CommandPrompt, Overlay, OverlayType};
use crate::utils;
use crate::textobject::{Anchor, TextObject, Kind, Offset};

// FIXME: Temporary replacement for the RustBox method `print_char` and this source's reliance on it.
// Such that: rb.print_char(offset, height + 1, RustBoxStyle::empty(), Color::White, Color::Black, ch);
macro_rules! print_char {
    ($dst:expr, $col:expr, $row:expr, $ch:expr) => {
        let cursor = TerminalCursor::new();
        cursor.hide().unwrap();
        cursor.goto($col, $row).unwrap();
        write!($dst, "{}{}", $ch, Attribute::Reset).unwrap();
        // crossterm::terminal().write($ch).unwrap();
        // crossterm::terminal().write(Attribute::Reset).unwrap(); // Clear color and style settings
    };
}

/// A View is an abstract Window (into a Buffer).
///
/// It draws a portion of a Buffer to a `UIBuffer` which in turn is drawn to the
/// screen. It maintains the status bar for the current view, the "dirty status"
/// which is whether the buffer has been modified or not and a number of other
/// pieces of information.
pub struct View<'v> {
    pub buffer: Arc<Mutex<Buffer>>,
    pub last_buffer: Option<Arc<Mutex<Buffer>>>,
    pub overlay: Option<Box<Overlay + 'v>>,

    height: u16,
    width: u16,

    /// First character of the top line to be displayed
    top_line: Mark,

    /// Index into the top_line - used for horizontal scrolling
    left_col: u16,

    /// The current View's cursor - a reference into the Buffer
    cursor: Mark,

    /// Number of lines from the top/bottom of the View after which vertical
    /// scrolling begins.
    threshold: u16,

    /// Message to be displayed in the status bar along with the time it
    /// was displayed.
    message: Option<(String, SystemTime)>,
}

impl<'v> View<'v> {

    pub fn new(buffer: Arc<Mutex<Buffer>>, width: u16, height: u16) -> View<'v> {
        let cursor = Mark::Cursor(0);
        let top_line = Mark::DisplayMark(0);

        {
            let mut b = buffer.lock().unwrap();

            b.set_mark(cursor, 0);
            b.set_mark(top_line, 0);
        }

        View {
            buffer: buffer,
            last_buffer: None,
            top_line: top_line,
            left_col: 0,
            cursor: cursor,
            overlay: None,
            threshold: 5,
            message: None,
            height: height,
            width: width,
        }
    }

    pub fn set_buffer(&mut self, buffer: Arc<Mutex<Buffer>>) {
        self.last_buffer = Some(self.buffer.clone());

        {
            let mut b = buffer.lock().unwrap();

            b.set_mark(self.cursor, 0);
            b.set_mark(self.top_line, 0);
        }

        self.buffer = buffer;
    }

    pub fn switch_last_buffer(&mut self) {
        let buffer = self.buffer.clone();
        let last_buffer = match self.last_buffer.clone() {
            Some(buf) => buf,
            None => return
        };

        self.buffer = last_buffer;
        self.last_buffer = Some(buffer);
    }

    /// Get the height of the View.
    ///
    /// This is the height of the UIBuffer minus the status bar height.
    pub fn get_height(&self) -> u16 {
        self.height - 1
    }

    /// Get the width of the View.
    pub fn get_width(&self) -> u16 {
        self.width + 1 // Everything from Iota seems to believe the width of the window is less than it is...
    }

    /// Resize the view
    ///
    /// This involves simply changing the size of the associated UIBuffer
    pub fn resize(&mut self, width: u16, height: u16) {
        self.height = height;
        self.width = width;
    }

    /// Clear the buffer
    ///
    /// Fills every cell in the UIBuffer with the space (' ') char.
    pub fn clear(&mut self, rb: &mut Crossterm) {
        // for row in 0..self.height {
        //     for col in 0..self.width {
        //         // rb.print_char(col, row, RustBoxStyle::empty(), Color::White, Color::Black, ' ');
        //         print_char!(col, row, ' ');
        //     }
        // }
        rb.terminal().clear(crossterm::ClearType::All).unwrap();
    }

    pub fn draw(&mut self, rb: &mut Crossterm) {
        self.clear(rb);
        {
            let buffer = self.buffer.lock().unwrap();
            let height = self.get_height() - 1;
            let width = self.get_width() - 1;

            // FIXME: don't use unwrap here
            //        This will fail if for some reason the buffer doesnt have
            //        the top_line mark
            let mut lines = buffer.lines_from(self.top_line).unwrap().take(height as usize);
            for y_position in 0..height {
                let line = lines.next().unwrap_or_else(Vec::new);
                draw_line(rb, &line, y_position, self.left_col);
            }

        }

        self.draw_status(rb);

        match self.overlay {
            None => self.draw_cursor(rb),
            Some(ref mut overlay) => {
                overlay.draw(rb);
                overlay.draw_cursor(rb);
            }
        }
    }

    #[cfg_attr(feature="clippy", allow(needless_range_loop))]
    fn draw_status(&mut self, rb: &mut Crossterm) {
        let buffer = self.buffer.lock().unwrap();
        let buffer_name = buffer.file_name();

        let mut cursor_status = buffer.get_mark_display_coords(self.cursor).unwrap_or((0,0));
        cursor_status = (cursor_status.0 + 1, cursor_status.1 + 1);

        let mut status_text: String = format!("{} [{}]", Colored::Bg(Color::Rgb{r:0,g:0,b:175}), buffer_name);

        let width = self.get_width();
        let height = self.get_height();

        let stdout = std::io::stdout();
        let mut out = stdout.lock();

        if buffer.dirty {
            status_text.push_str("●");
        }

        status_text.push_str(&format!(" ({}, {})", cursor_status.0, cursor_status.1));

        let len = status_text.len() as u16;
        for _ in 0..width-len {
            status_text.push(' ');
        }
        print_char!(out, 0, height, status_text);

        // For the message at the very bottom of the window
        if let Some((ref message, _time)) = self.message {
            print_char!(out, 0, height + 1, message);
        }
    }

    fn draw_cursor(&mut self, rb: &mut Crossterm) {
        let buffer = self.buffer.lock().unwrap();
        if let Some(top_line) = buffer.get_mark_display_coords(self.top_line) {
            if let Some((x, y)) = buffer.get_mark_display_coords(self.cursor) {
                // rb.set_cursor((x - self.left_col) as isize, y as isize - top_line.1 as isize);
                rb.cursor().show().unwrap();
                rb.cursor().goto((x - self.left_col as usize) as u16, (y - top_line.1) as u16).unwrap();
            }
        }
    }

    pub fn set_overlay(&mut self, overlay_type: OverlayType) {
        match overlay_type {
            OverlayType::CommandPrompt => {
                self.overlay = Some(Box::new(CommandPrompt::new()));
            }
        }
    }

    /// Display the given message
    pub fn show_message(&mut self, message: String) {
        self.message = Some((message, SystemTime::now()));
    }

    /// Clear the currently displayed message if it has been there for 5 or more seconds
    ///
    /// Does nothing if there is no message, or of the message has been there for
    /// less that five seconds.
    pub fn maybe_clear_message(&mut self) {
        let mut clear_message = false;
        if let Some((_, time)) = self.message {
            if let Ok(elapsed) = time.elapsed() {
                if elapsed.as_secs() >= 5 {
                    clear_message = true;
                }
            }
        }
        if clear_message {
            self.message = None;
        }
    }

    pub fn move_mark(&mut self, mark: Mark, object: TextObject) {
        self.buffer.lock().unwrap().set_mark_to_object(mark, object);
        self.maybe_move_screen();
    }

    /// Update the top_line mark if necessary to keep the cursor on the screen.
    fn maybe_move_screen(&mut self) {
        let mut buffer = self.buffer.lock().unwrap();
        if let (Some(cursor), Some((_, top_line))) = (buffer.get_mark_display_coords(self.cursor),
                                                      buffer.get_mark_display_coords(self.top_line)) {

            let width  = self.get_width()  - self.threshold;
            let height = self.get_height() - self.threshold;

            //left-right shifting
            self.left_col = match cursor.0 as isize - self.left_col as isize {
                x_offset if x_offset < self.threshold as isize => {
                    cmp::max(0isize, self.left_col as isize - (self.threshold as isize - x_offset)) as u16
                }
                x_offset if x_offset >= width as isize => {
                    self.left_col + (x_offset as u16 - width + 1)
                }
                _ => { self.left_col }
            };

            //up-down shifting
            match cursor.1 as isize - top_line as isize {
                y_offset if y_offset < self.threshold as isize && top_line > 0 => {
                    let amount = (self.threshold as isize - y_offset) as usize;
                    let obj = TextObject {
                        kind: Kind::Line(Anchor::Same),
                        offset: Offset::Backward(amount, self.top_line)
                    };
                    buffer.set_mark_to_object(self.top_line, obj);
                }
                y_offset if y_offset >= height as isize => {
                    let amount = (y_offset - height as isize + 1) as usize;
                    let obj = TextObject {
                        kind: Kind::Line(Anchor::Same),
                        offset: Offset::Forward(amount, self.top_line)
                    };
                    buffer.set_mark_to_object(self.top_line, obj);
                }
                _ => { }
            }
        }
    }

    // Delete chars from the first index of object to the last index of object
    pub fn delete_object(&mut self, object: TextObject) {
        self.buffer.lock().unwrap().remove_object(object);
    }

    pub fn delete_from_mark_to_object(&mut self, mark: Mark, object: TextObject) {
        let mut buffer = self.buffer.lock().unwrap();
        if let Some(mark_pos) = buffer.get_object_index(object) {
            if let Some(midx) = buffer.get_mark_idx(mark) {
                buffer.remove_from_mark_to_object(mark, object);
                buffer.set_mark(mark, cmp::min(mark_pos.absolute, midx));
            }
        }
    }

    /// Insert a chacter into the buffer & update cursor position accordingly.
    pub fn insert_char(&mut self, ch: char) {
        self.buffer.lock().unwrap().insert_char(self.cursor, ch as u8);
        // NOTE: the last param to char_width here may not be correct
        if let Some(ch_width) = utils::char_width(ch, false, 4, 1) {
            let obj = TextObject {
                kind: Kind::Char,
                offset: Offset::Forward(ch_width, Mark::Cursor(0))
            };
            self.move_mark(Mark::Cursor(0), obj)
        }
    }

    pub fn undo(&mut self) {
        {
            let mut buffer = self.buffer.lock().unwrap();
            let point = if let Some(transaction) = buffer.undo() { transaction.end_point }
                        else { return; };
            buffer.set_mark(self.cursor, point);
        }
        self.maybe_move_screen();
    }

    pub fn redo(&mut self) {
        {
            let mut buffer = self.buffer.lock().unwrap();
            let point = if let Some(transaction) = buffer.redo() { transaction.end_point }
                        else { return; };
            buffer.set_mark(self.cursor, point);
        }
        self.maybe_move_screen();
    }

    fn save_buffer(&mut self) {
        let buffer = self.buffer.lock().unwrap();
        let path = match buffer.file_path {
            Some(ref p) => Cow::Borrowed(p),
            None => {
                // NOTE: this should never happen, as the file path
                // should have been set inside the try_save_buffer method.
                //
                // If this runs, it probably means save_buffer has been called
                // directly, rather than try_save_buffer.
                //
                // TODO: ask the user to submit a bug report on how they hit this.
                Cow::Owned(PathBuf::from("untitled"))
            },
        };
        let tmpdir = match TempDir::new_in(&Path::new("."), "iota") {
            Ok(d) => d,
            Err(e) => panic!("file error: {}", e)
        };

        let tmppath = tmpdir.path().join(Path::new("tmpfile"));
        let mut file = match File::create(&tmppath) {
            Ok(f) => f,
            Err(e) => {
                panic!("file error: {}", e)
            }
        };

        //TODO (lee): Is iteration still necessary in this format?
        for line in buffer.lines() {
            let result = file.write_all(&*line);

            if result.is_err() {
                // TODO(greg): figure out what to do here.
                panic!("Something went wrong while writing the file");
            }
        }

        if let Err(e) = rename(&tmppath, &*path) {
            panic!("file error: {}", e);
        }
    }

    pub fn try_save_buffer(&mut self) {
        let mut should_save = false;
        {
            let buffer = self.buffer.lock().unwrap();

            match buffer.file_path {
                Some(_) => { should_save = true; }
                None => {
                    self.message = Some(("No file name".into(), SystemTime::now()));
                }
            }
        }

        if should_save {
            self.save_buffer();
            let mut buffer = self.buffer.lock().unwrap();
            buffer.dirty = false;
        }
    }

    /// Whether or not the current buffer has unsaved changes
    pub fn buffer_is_dirty(&mut self) -> bool {
        self.buffer.lock().unwrap().dirty
    }

}

pub fn draw_line(rb: &mut Crossterm, line: &[u8], idx: u16, left: u16) {
    let width = rb.terminal().terminal_size().0 - 1;
    let mut x: u16 = 0;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    // In Iota, this function would attempt to draw every character individually, but crossterm is better
    // at drawing more text than a character at a time. It prefers being buffered. This is the solution.
    let mut formatted_line = String::new(); // Line after applying tabs and characters that fit within view

    for ch in line.iter().skip(left as usize) {
        let ch = *ch as char;
        match ch {
            '\t' => {
                let w = 4 - x % 4;
                for _ in 0..w {
                    // rb.print_char(x, idx, RustBoxStyle::empty(), Color::White, Color::Black, ' ');
                    // print_char!(out, x, idx, ' ');
                    formatted_line.push(' ');
                    x += 1;
                }
            }
            '\n' => {}
            _ => {
                // rb.print_char(x, idx, RustBoxStyle::empty(), Color::White, Color::Black, ch);
                // print_char!(out, x, idx, ch);
                formatted_line.push(ch);
                x += UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
            }
        }
        if x >= width { // When line has run to the end of the view
            break;
        }
    }

    // Replace any cells after end of line with ' '
    // while x < width {
    //     // rb.print_char(x, idx, RustBoxStyle::empty(), Color::White, Color::Black, ' ');
    //     print_char!(out, x, idx, ' ');
    //     x += 1;
    // }
    // if x < width { // Again, crossterm is a little higher-level
    //     formatted_line.push('\n');
    // }

    print_char!(out, 0, idx, formatted_line); // Write the entire line

    // If the line is too long to fit on the screen, show an indicator
    let indicator = if line.len() > (width + left) as usize { '→' } else { ' ' };
    // rb.print_char(width, idx, RustBoxStyle::empty(), Color::White, Color::Black, indicator);
    print_char!(out, width, idx, indicator);
}

#[cfg(test)]
mod tests {

    use std::sync::{Arc, Mutex};
    use std::rc::Rc;

    use crate::view::View;
    use crate::buffer::Buffer;

    fn setup_view(testcase: &'static str) -> View {
        let buffer = Arc::new(Mutex::new(Buffer::new()));
        let mut view = View::new(buffer.clone(), 50, 50);
        for ch in testcase.chars() {
            view.insert_char(ch);
        }

        let mut buffer = buffer.lock().unwrap();
        buffer.set_mark(view.cursor, 0);
        view
    }

    #[test]
    fn test_insert_char() {
        let mut view = setup_view("test\nsecond");
        view.insert_char('t');

        {
            let mut buffer = view.buffer.lock().unwrap();
            assert_eq!(buffer.lines().next().unwrap(), b"ttest\n");
        }
    }
}
