**tree2tui** is a command line tool to build a Terminal UI (*TUI*) for `tree` like command,
aiming to enable folding/expanding tree nodes for large/complex tree.

## Usage
```shell
$ tree | tree2tui

$ cargo tree | tree2tui

// --cargo|-c    enalbe folding/expanding duplicate nodes of "cargo tree"
$ cargo tree | tree2tui --cargo
```
