#![cfg(not(test))]

use structopt;

use libc;
use rustc_serialize;
use crossterm;
use credits;

use std::io::stdin;
// use docopt::Docopt;
use credits::{
    Editor, Input,
    StandardMode, NormalMode, EmacsMode,
    Mode,
};

use crossterm::Crossterm;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "iota")]
struct Opt {
    #[structopt(name = "FILE")]
    arg_filename: Option<String>,
    /// Start Credits with Emacs-like mode
    #[structopt(long = "emacs")]
    flag_emacs: bool,
    /// Start Credits with Vi-like modes
    #[structopt(long = "vi")]
    flag_vi: bool,
}

fn is_atty(fileno: libc::c_int) -> bool {
    // FIXME: find a way to do this without unsafe
    //        std::io doesn't allow for this, currently
    unsafe { libc::isatty(fileno) != 0 }
}

fn main() {
    // let args: Args = Docopt::new(USAGE)
    //                         .and_then(|d| d.decode())
    //                         .unwrap_or_else(|e| e.exit());
    let args = Opt::from_args();

    let stdin_is_atty = is_atty(libc::STDIN_FILENO);
    // let stderr_is_atty = is_atty(libc::STDERR_FILENO);

    // editor source - either a filename or stdin
    let source = if stdin_is_atty {
        Input::Filename(args.arg_filename)
    } else {
        Input::Stdin(stdin())
    };


    // initialise rustbox
    // let rb = match RustBox::init(InitOptions{
    //     buffer_stderr: stderr_is_atty,
    //     input_mode: InputMode::Esc,
    //     output_mode: OutputMode::EightBit,
    // }) {
    //     Result::Ok(v) => v,
    //     Result::Err(e) => panic!("{}", e),
    // };
    let ct = Crossterm::new();

    // initialise the editor mode
    let mode: Box<Mode> = if args.flag_vi {
        Box::new(NormalMode::new())
    } else if args.flag_emacs {
        Box::new(EmacsMode::new())    
    } else {
        Box::new(StandardMode::new())
    };

    // start the editor
    let mut editor = Editor::new(source, mode, ct);
    editor.start();
}
