extern crate termion;
extern crate rand;
extern crate itertools;

#[macro_use]
extern crate clap;

use rand::{thread_rng, sample};

use itertools::Itertools;

use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead, MouseTerminal};
use termion::raw::IntoRawMode;
use termion::{color, style, cursor};

use std::io::{Write};

#[derive(Clone, Copy)]
struct Tile {
    kind: TileType,
    uncovered: bool,
    marked: MarkType
}

impl Tile {
    fn new(kind: TileType) -> Self {
        Tile {
            kind: kind, uncovered: false, marked: MarkType::No
        }
    }
}

#[derive(Clone, Copy)]
enum TileType {
    Safe(u8),
    Mine
}

#[derive(Clone, Copy, PartialEq)]
enum MarkType {
    No, Yes, Uncertain
}

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Play, Win, Lose
}

struct TileArray {
    data: Vec<Box<Tile>>,

    width: usize,
    height: usize,
    cursor_x: usize,
    cursor_y: usize,
    array_screen_height: usize,

    game_state: GameState,

    num_mines: usize
}

impl TileArray {
    fn new(config: GameConfig) -> Self {
        TileArray {
            data: vec![Box::new(Tile::new(TileType::Safe(0))); config.width * config.height],
            width: config.width,
            height: config.height,
            cursor_x: 0, cursor_y: 0,
            array_screen_height: 2,
            game_state: GameState::Play,
            num_mines: config.mines
        }
    }

    fn get_tile_type(&self, x: usize, y: usize) -> TileType {
        self.data[x + y * self.width].kind
    }

    fn set_tile_type(&mut self, x: usize, y: usize, kind: TileType) {
        self.data[x + y * self.width].kind = kind;
    }

    fn is_tile_uncovered(&self, x: usize, y: usize) -> bool {
        self.data[x + y * self.width].uncovered
    }

    fn set_tile_uncovered(&mut self, x: usize, y: usize, uncover: bool) {
        self.data[x + y * self.width].uncovered = uncover;
    }

    fn get_tile_mark(&self, x: usize, y: usize) -> MarkType {
        self.data[x + y * self.width].marked
    }

    fn set_tile_mark(&mut self, x: usize, y: usize, marked: MarkType) {
        self.data[x + y * self.width].marked = marked;
    }

    fn move_cursor(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.cursor_x = x;
            self.cursor_y = y;
        }
    }

    fn move_cursor_x(&mut self, x: usize) {
        let y = self.cursor_y;
        self.move_cursor(x, y);
    }

    fn move_cursor_y(&mut self, y: usize) {
        let x = self.cursor_x;
        self.move_cursor(x, y);
    }

    fn setup(&mut self) {
        let mut rng = rand::thread_rng();
        let list = (0..self.width).cartesian_product((0..self.height));
        let sample = rand::sample(&mut rng, list, self.num_mines);
        for i in 0..self.data.len() {
            let x = i % self.width;
            let y = i / self.width;
            self.set_tile_mark(x, y, MarkType::No);
            self.set_tile_uncovered(x, y, false);
            let kind = match sample.iter().find(|&&item| item == (x, y)) {
                Some(_) => TileType::Mine,
                None => TileType::Safe(0)
            };
            self.set_tile_type(x, y, kind);
        }
        for i in 0..self.data.len() {
            let cur_x = i % self.width;
            let cur_y = i / self.width;
            let kind = self.get_tile_type(cur_x, cur_y);
            if let TileType::Safe(_) = kind {
                let offsets: [(i8, i8); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
                let adj_mines = offsets.iter()
                    .map(|&(ox, oy)| { (cur_x as i8 + ox, cur_y as i8 + oy) })
                    .filter(|&(x, y)| { x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height })
                    .filter(|&(x, y)| {
                        if let TileType::Mine = 
                            self.get_tile_type(x as usize, y as usize) {true} else {false}
                    })
                    .count();
                self.set_tile_type(cur_x, cur_y, TileType::Safe(adj_mines as u8));
            }
        }
    }

    fn render(&self, stdout: &mut std::io::Stdout) {
        print!("{}{}Minesweeper! (Press q to quit)\n\r\n\r", termion::clear::All, termion::cursor::Goto(1, 1));
        for (i, item) in self.data.iter().enumerate() {
            if item.uncovered {
                match item.kind {
                    TileType::Safe(0) => print!("░"),
                    TileType::Safe(n) => print!("{}", n),
                    TileType::Mine => print!("x")
                }
            }
            else { 
                match item.marked {
                    MarkType::No => print!("▓"),
                    MarkType::Yes => print!("✓"),
                    MarkType::Uncertain => print!("?")
                }
            }
            if i % self.width == self.width - 1 { print!("\n\r"); }
        }
        print!("\n\r");
        match self.game_state {
            GameState::Win => print!("Congratulations! You win the game! (Press r to restart, q to quit)\n\r"),
            GameState::Lose => print!("Game Over! (Press r to restart, q to quit)\n\r"),
            GameState::Play => {}
        }
        write!(stdout, "{}", cursor::Goto(
            self.cursor_x as u16 + 1, 
            self.array_screen_height as u16 + self.cursor_y as u16 + 1
        )).unwrap();
    }

    fn uncover_tile(&mut self, x: usize, y: usize, deduction: bool) {
        let mark = self.get_tile_mark(x, y);
        if mark == MarkType::Yes || mark == MarkType::Uncertain { return; }
        if !deduction && self.is_tile_uncovered(x, y) { return; }
        self.set_tile_uncovered(x, y, true);
        let kind = self.get_tile_type(x, y);
        if let TileType::Safe(n) = kind {
            let offsets: [(i8, i8); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
            let adj_tiles: Vec<(usize, usize)> = offsets.iter()
                .map(|&(ox, oy)| (x as i8 + ox, y as i8 + oy))
                .filter(|&(x, y)| x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height)
                .map(|(x, y)| (x as usize, y as usize))
                .collect();
            let adj_mines_count = adj_tiles.iter().cloned()
                .filter(|&(x, y)| self.get_tile_mark(x, y) == MarkType::Yes)
                .count();
            if n as usize == adj_mines_count {
                let unmarked_tiles_pos: Vec<(usize, usize)> = adj_tiles.iter().cloned()
                    .filter(|&(x, y)| self.get_tile_mark(x, y) == MarkType::No).collect();
                for (x, y) in unmarked_tiles_pos {
                    self.uncover_tile(x, y, false);
                }
            }
        }
        else {
            self.game_state = GameState::Lose;
        }
    }

    fn mark_tile(&mut self, x: usize, y: usize) {
        match self.get_tile_mark(x, y) {
            MarkType::No => self.set_tile_mark(x, y, MarkType::Yes),
            MarkType::Yes => self.set_tile_mark(x, y, MarkType::Uncertain),
            MarkType::Uncertain => self.set_tile_mark(x, y, MarkType::No)
        }
    }

    fn check_win_condition(&mut self) {
        let tiles_left = self.data.iter()
            .filter(|&data| !data.uncovered).count();
        let num_marked = self.data.iter()
            .filter(|&data| data.marked == MarkType::Yes).count();
        if tiles_left == self.num_mines && num_marked == self.num_mines {
            self.game_state = GameState::Win;
        }
    }

    fn handle_events(&mut self, stdin: &mut std::io::Stdin, stdout: &mut std::io::Stdout) {
        for c in stdin.events() {
            let event = c.unwrap();
            let cursor_x = self.cursor_x;
            let cursor_y = self.cursor_y;
            match self.game_state {
                GameState::Lose | GameState::Win => match event {
                    Event::Key(Key::Char('q')) => break,
                    Event::Key(Key::Char('r')) => {
                        self.setup();
                        self.game_state = GameState::Play;
                    },
                    _ => {}
                },
                GameState::Play => match event {
                    Event::Key(Key::Char('q')) => {
                        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
                        break;
                    },
                    Event::Key(Key::Char('h')) => if cursor_x > 0 { self.move_cursor_x(cursor_x - 1) },
                    Event::Key(Key::Char('l')) => self.move_cursor_x(cursor_x + 1),
                    Event::Key(Key::Char('k')) => if cursor_y > 0 { self.move_cursor_y(cursor_y - 1) },
                    Event::Key(Key::Char('j')) => self.move_cursor_y(cursor_y + 1),
                    Event::Key(Key::Char(' ')) | Event::Key(Key::Char('f')) 
                        => self.uncover_tile(cursor_x, cursor_y, true),
                    Event::Key(Key::Char('d'))
                        => self.mark_tile(cursor_x, cursor_y),
                    _ => {}
                }
            } 
            self.check_win_condition();
            self.render(stdout);
            stdout.flush().unwrap();
        }
    }
}

struct GameConfig {
    width: usize,
    height: usize,
    mines: usize
}
fn main() {
    let matches = clap_app!(minesweeper =>
        (version: "1.0")
        (author: "Phillip Chang <lasagnaphil@snu.ac.kr>")
        (about: "Minesweeper implementation in Rust")
        (@arg WIDTH: -w --width +takes_value requires[HEIGHT MINES] "Sets the width of the board")
        (@arg HEIGHT: -h --height +takes_value requires[WIDTH MINES] "Sets the height of the board")
        (@arg MINES: -m --mines +takes_value requires[WIDTH HEIGHT] "Sets the number of mines in the board")
        (@arg DIFFICULTY: -d --difficulty +takes_value possible_value[easy medium hard]
            conflicts_with[WIDTH HEIGHT MINES] "Sets the difficulty of the game (easy, medium, or hard)")
    ).get_matches();

    let gameConfig = match matches.value_of("DIFFICULTY") {
        Some("easy") => GameConfig { width: 8, height: 8, mines: 10 },
        Some("medium") => GameConfig { width: 16, height: 16, mines: 40 },
        Some("hard") => GameConfig { width: 30, height: 16, mines: 99 },
        Some(_) => panic!("Invalid difficulty!"),
        None => {
            GameConfig {
                width: value_t!(matches, "WIDTH", usize).unwrap_or_else(|e| e.exit()),
                height: value_t!(matches, "HEIGHT", usize).unwrap_or_else(|e| e.exit()),
                mines: value_t!(matches, "MINES", usize).unwrap_or_else(|e| e.exit())
            }
        }
    };

    let mut stdin = std::io::stdin();
    let mut stdout = MouseTerminal::from(
        std::io::stdout().into_raw_mode().unwrap());

    let mut tile_array = TileArray::new(gameConfig);

    tile_array.setup();
    tile_array.render(&mut stdout);
    tile_array.handle_events(&mut stdin, &mut stdout);
}
