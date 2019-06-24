use std::path::PathBuf;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;
use std::collections::HashMap;

// use rustbox::{RustBox, Event};
use crossterm::{InputEvent, KeyEvent, RawScreen, Crossterm};

use input::Input;
use view::View;
use modes::{Mode, ModeType, InsertMode, NormalMode};
use buffer::Buffer;
use command::Command;
use command::{Action, BuilderEvent, BuilderArgs, Operation, Instruction};


type EditorCommand = fn(Option<BuilderArgs>) -> Command;
lazy_static! {
    pub static ref ALL_COMMANDS: HashMap<&'static str, EditorCommand> = {
        let mut map: HashMap<&'static str, EditorCommand> = HashMap::new();

        map.insert("editor::quit", Command::exit_editor);
        map.insert("editor::save_buffer", Command::save_buffer);
        map.insert("editor::noop", Command::noop);

        map.insert("editor::undo", Command::undo);
        map.insert("editor::redo", Command::redo);
        map.insert("editor::set_mode", Command::set_mode);

        map.insert("editor::set_overlay", Command::set_overlay);

        map.insert("buffer::move_cursor", Command::move_cursor);
        map.insert("buffer::insert_char", Command::insert_char);
        map.insert("buffer::insert_tab", Command::insert_tab);
        map.insert("buffer::delete_char", Command::delete_char);


        map
    };
}

/// The main Editor structure
///
/// This is the top-most structure in Iota.
pub struct Editor<'e> {
    buffers: Vec<Arc<Mutex<Buffer>>>,
    view: View<'e>,
    running: bool,
    rb: Crossterm,
    mode: Box<Mode + 'e>,

    command_queue: Receiver<Command>,
    command_sender: Sender<Command>,
    
    just_attempted_exit: bool,
}

impl<'e> Editor<'e> {

    /// Create a new Editor instance from the given source
    pub fn new(source: Input, mode: Box<Mode + 'e>, rb: Crossterm) -> Editor<'e> {
        let (width, height) = rb.terminal().terminal_size();

        let (snd, recv) = channel();

        let mut buffers = Vec::new();

        let buffer = match source {
            Input::Filename(path) => {
                match path {
                    Some(path) => Buffer::from(PathBuf::from(path)),
                    None       => Buffer::new(),
                }
            },
            Input::Stdin(reader) => {
                Buffer::from(reader)
            }
        };
        buffers.push(Arc::new(Mutex::new(buffer)));

        let view = View::new(buffers[0].clone(), width, height);

        Editor {
            buffers: buffers,
            view: view,
            running: true,
            rb: rb,
            mode: mode,

            command_queue: recv,
            command_sender: snd,
            
            just_attempted_exit: false,
        }
    }

    /// Handle key events
    ///
    /// Key events can be handled in an Overlay, OR in the current Mode.
    ///
    /// If there is an active Overlay, the key event is sent there, which gives
    /// back an OverlayEvent. We then parse this OverlayEvent and determine if
    /// the Overlay is finished and can be cleared. The response from the
    /// Overlay is then converted to a Command and sent off to be handled.
    ///
    /// If there is no active Overlay, the key event is sent to the current
    /// Mode, which returns a Command which we dispatch to handle_command.
    fn handle_key_event(&mut self, event: KeyEvent) {
        let command = match self.view.overlay {
            None                  => self.mode.handle_key_event(event),
            Some(ref mut overlay) => overlay.handle_key_event(event),
        };

        if let BuilderEvent::Complete(c) = command {
            self.view.overlay = None;
            self.view.clear(&mut self.rb);

            match ALL_COMMANDS.get(&*c.command_name) {
                Some(cmd) => {
                    let cmd = cmd(c.args);
                    let _ = self.command_sender.send(cmd);
                }
                None => {
                    panic!("Unknown command: {}", c.command_name);
                }
            }

            // let _ = self.command_sender.send(c);
        }
    }

    /// Handle resize events
    ///
    /// width and height represent the new height of the window.
    fn handle_resize_event(&mut self, width: u16, height: u16) {
        self.view.resize(width, height);
        self.draw();
    }

    /// Draw the current view to the frontend
    fn draw(&mut self) {
        self.view.draw(&mut self.rb);
    }

    /// Handle the given command, performing the associated action
    fn handle_command(&mut self, command: Command) {
        let repeat = if command.number > 0 {
            command.number
        } else { 1 };
        for _ in 0..repeat {
            match command.action {
            	Action::Instruction(Instruction::ExitEditor) => self.handle_instruction(command.clone()),
                Action::Instruction(_) => {
                	self.handle_instruction(command.clone());
                	// To keep the "Unsaved changes" message from preventing force quit:
                	match command.action {
                		Action::Instruction(Instruction::ShowMessage(_)) => {},
                		_ => self.just_attempted_exit = false,
                	}
                }
                Action::Operation(_) => {
                	self.handle_operation(command.clone());
                	self.just_attempted_exit = false;
                }
            }
        }
        self.draw(); // Redraw after updating
    }


    fn handle_instruction(&mut self, command: Command) {
        match command.action {
            Action::Instruction(Instruction::SaveBuffer) => { self.view.try_save_buffer() }
            Action::Instruction(Instruction::ExitEditor) => {
                if self.view.buffer_is_dirty() {
                	if self.just_attempted_exit {
                		self.running = false; // Allow "force quit"
                	} else {
                		self.just_attempted_exit = true;
                		
                		let args = BuilderArgs::new().with_str("Unsaved changes".into());
                		let _ = self.command_sender.send(Command::show_message(Some(args)));
                	}
                } else {
                    self.running = false;
                }
            }
            Action::Instruction(Instruction::SetMark(mark)) => {
                if let Some(object) = command.object {
                    self.view.move_mark(mark, object)
                }
            }
            Action::Instruction(Instruction::SetOverlay(overlay_type)) => {
                self.view.set_overlay(overlay_type)
            }
            Action::Instruction(Instruction::SetMode(mode)) => {
                match mode {
                    ModeType::Insert => { self.mode = Box::new(InsertMode::new()) }
                    ModeType::Normal => { self.mode = Box::new(NormalMode::new()) }
                }
            }
            Action::Instruction(Instruction::SwitchToLastBuffer) => {
                self.view.switch_last_buffer();
                self.view.clear(&mut self.rb);
            }
            Action::Instruction(Instruction::ShowMessage(msg)) => {
                self.view.show_message(msg)
            }

            _ => {}
        }
    }

    fn handle_operation(&mut self, command: Command) {
        match command.action {
            Action::Operation(Operation::Insert(c)) => {
                for _ in 0..command.number {
                    self.view.insert_char(c)
                }
            }
            Action::Operation(Operation::DeleteObject) => {
                if let Some(obj) = command.object {
                    self.view.delete_object(obj);
                }
            }
            Action::Operation(Operation::DeleteFromMark(m)) => {
                if command.object.is_some() {
                    self.view.delete_from_mark_to_object(m, command.object.unwrap())
                }
            }
            Action::Operation(Operation::Undo) => { self.view.undo() }
            Action::Operation(Operation::Redo) => { self.view.redo() }

            Action::Instruction(_) => {}
        }
    }

    /// Start Iota!
    pub fn start(&mut self) {
        if let Ok(_raw) = RawScreen::into_raw_mode() { // Keep terminal from processing events for us
            let mut term_size = self.rb.terminal().terminal_size();

            self.draw(); // Draw once for the first time

            let mut sync_stdin = self.rb.input().read_sync();
            while self.running {
                // self.draw();
                // self.rb.terminal().clear(crossterm::ClearType::All);
                self.view.maybe_clear_message();

                match sync_stdin.next() {
                    // FIXME: Update this when it gets fully added to crossterm
                    // Some(InputEvent::Resize) => self.handle_resize_event(width, height),
                    Some(InputEvent::Keyboard(key_event)) => self.handle_key_event(key_event),
                    _ => {}
                }

                // Update view size by polling view (see FIXME above)
                if term_size != self.rb.terminal().terminal_size() { // Outdated, let's update
                    term_size = self.rb.terminal().terminal_size();
                    self.handle_resize_event(term_size.0, term_size.1);
                }

                while let Ok(message) = self.command_queue.try_recv() {
                    self.handle_command(message)
                }
            }
        } else {
            panic!("Could not start application with raw mode. Unsupported terminal?");
        }
    }
}
