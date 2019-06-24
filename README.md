# Credits

Credits is a CLI text-editor for cross-platform edits. The focus of Credits is to offer a native
command-line text editing software for all mainstream operating systems.

Here's what it looks like right now, editing itself.

![Screenshot](screenshot.png)

Credits is a fork of the popular, but dead, Iota text editor written in pure Rust. This fork currently aims to do several things before fully changing courses:

 - Replace rustbox (Unix-only) backend with crossterm (done)
 - Bring codebase up to date with latest Rust (changing dependencies out) (wip)
 - Refactor and aim to remove as much code as possible (slim the codebase down) (up next)
 - Fix bugs from iota issues list (wip)

### Course Change
When this fork has finished the above, it will begin implementing the following:
 - PEG AST-based syntax highlighting (using pest parser)
 - A single, standard control scheme with support for loading custom, scriptable control schemes (getting rid of built-in Emacs and Vi controls)

## Building
### To Install
Clone (or download) the project and install it using cargo:
```bash
git clone https://github.com/asmoaesl/credits.git
cd credits
cargo install --path .
```
Now Credits is installed on your computer and available in your PATH.

**NOTE:** Credits needs to be built using the nightly toolchain for now, not stable.<br>
Run the following commands - `$ rustup install nightly` following which run - `$ rustup override set nightly `.<br>
[Rustup](https://github.com/rust-lang-nursery/rustup.rs) is very useful for managing
multiple rust versions.

### Usage

To start the editor run `./target/release/credits /path/to/file.txt`. Or
simply `./target/release/credits` to open an empty buffer.

You can also create buffers from `stdin`.

```bash
# open a buffer with the output of `ifconfig`
ifconfig | ./target/release/credits
```

You can move the cursor around with the arrow keys.

The following keyboard bindings are also available:

- `Ctrl-s` save
- `Ctrl-q` quit
- `Ctrl-z` undo
- `Ctrl-y` redo

Credits currently supports both Vi and Emacs style keybindings for simple movement.

You can enable Vi style keybindings by using the `--vi` flag when starting Iota.
The vi-style modes are in the early stages, and not all functionality is there
just yet. The following works:

- while in normal mode:
    - `k` move up
    - `j` move down
    - `l` move forwards
    - `h` move backwards
    - `w` move one word forward
    - `b` move one word backward
    - `0` move to start of line
    - `$` move to end of line
    - `d` delete
    - `u` undo
    - `r` redo
    - `i` insert mode
    - `:q` quit
    - `:w` save
- while in insert mode:
    - `ESC` normal mode

Alternatively, you can use the following emacs-style keys by using the `--emacs` flag:

- `Ctrl-p` move up
- `Ctrl-n` move down
- `Ctrl-b` move backwards
- `Ctrl-f` move forwards
- `Ctrl-a` move to start of line
- `Ctrl-e` move to end of line
- `Ctrl-d` delete forwards
- `Ctrl-h` delete backwards
- `Ctrl-x Ctrl-c` quit
- `Ctrl-x Ctrl-s` save
- `Ctrl-z` undo
- `Ctrl-y` redo
