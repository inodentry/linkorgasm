# linkorgasm - Tool for organizing files with symlinks

It manages a hierarchy of directories (think of them like tags or categories)
containing symlinks pointing to items located elsewhere.

Just select the source directory where the items are located and the tags
directory where to create symlinks and `linkorgasm` will show you a nice
terminal-based UI where you can select many items at once and categorize
them with a single keypress!

You can also open/preview the selected items from within `linkorgasm` using
a command of your choice.

`linkorgasm` is a powerful alternative to other file tagging/categorization
tools, which often use a special database or metadata format. By contrast,
`linkorgasm` requires no special support, as it just uses the filesystem. You
can access your beautifully-organized collection from your file manager or any
other app.

## Project status

The current version is usable and supports all advertised features, but feels
quite rough and could benefit from various usability improvements. See the
issue tracker. The code also needs to be cleaned up.

Tested on Linux, but should also work on other UNIX-like OSs. Windows not
supported yet.

## Compiling

Like any standard Rust program, `linkorgasm` uses `cargo`:

```
$ cargo run --release
```
