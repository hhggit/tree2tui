**tree2tui** is a command line tool to build a Terminal UI (*TUI*) for `tree` like command,
aiming to enable folding/expanding tree nodes for large/complex tree.

## Usage
```shell
$ tree | tree2tui

$ cargo tree | tree2tui

$ cargo tree | tree2tui --cargo
```
### options
```
USAGE:
    tree2tui [FLAGS] [OPTIONS]

FLAGS:
    -c, --cargo      enable folding/expanding duplicate nodes of "cargo tree"
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --data <data>      group selected from regex as tree's data, if not set using all data after
                           node
    -n, --node <node>      group selected from regex as tree's node [default: 1]
    -r, --regex <regex>    regex to capture the tree's node [default: [│\s]*([├└]─*\s*)]
```
![cargo tree](https://github.com/hhggit/tree2tui/blob/assets/screenshot-cargo_tree.jpg?raw=true)
