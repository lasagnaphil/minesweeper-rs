# minesweeper-rs

Minesweeper implementation in Rust

USAGE:

    minesweeper [OPTIONS]

FLAGS:

        --help       Prints help information        
    -V, --version    Prints version information

OPTIONS:

    -d, --difficulty <DIFFICULTY>    Sets the difficulty of the game (easy, medium, or hard)
                                     [values: easy, medium, hard]                                     
    -h, --height <HEIGHT>            Sets the height of the board
    -m, --mines <MINES>              Sets the number of mines in the board
    -w, --width <WIDTH>              Sets the width of the board

## Example

```bash
./minesweeper -d medium
```

```bash
./minesweeper -w 10 -h 10 -m 12
```
