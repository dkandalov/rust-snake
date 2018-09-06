extern crate libc;
extern crate rand;
#[macro_use] extern crate maplit;
use rand::Rng;
use rand::ChaChaRng;
use rand::SeedableRng;
use std::ffi::{CString};
use rand::rngs::EntropyRng;
use std::collections::HashSet;
use self::Direction::{Up, Down, Left, Right};

fn main() {
    unsafe {
        initscr();
        noecho();
        curs_set(0);
        halfdelay(2);

        let width = 20;
        let height = 10;
        let mut game = Game {
            width,
            height,
            snake: Snake::new(
                vec![cell(4, 0), cell(3, 0), cell(2, 0), cell(1, 0), cell(0, 0)],
                Right
            ),
            apples: Apples::create(width, height)
        };
        let window = newwin(game.height + 2, game.width + 2, 0, 0);

        let mut c = 0;
        while char::from(c as u8) != 'q' {

            game.draw(window);

            c = wgetch(window);
            let direction = match char::from(c as u8) {
                'i' => Some(Up),
                'j' => Some(Left),
                'k' => Some(Down),
                'l' => Some(Right),
                _ => Option::None
            };

            game = game.update(direction);
        }

        delwin(window);
        endwin();
    }
}

impl Game {
    fn draw(&self, window: *mut i8) {
        unsafe {
            wclear(window);
            box_(window, 0, 0);

            self.apples.cells.iter().for_each(|cell|
                mvwprintw(window, cell.y + 1, cell.x + 1, ".".to_c_str().as_ptr())
            );
            self.snake.tail().iter().for_each(|cell|
                mvwprintw(window, cell.y + 1, cell.x + 1, "o".to_c_str().as_ptr())
            );
            mvwprintw(window, self.snake.head().y + 1, self.snake.head().x + 1, "Q".to_c_str().as_ptr());

            if self.is_over() {
                mvwprintw(window, 0, 4, "Game is Over".to_c_str().as_ptr());
                mvwprintw(window, 1, 3, format!("Your score is {}", self.score()).as_str().to_c_str().as_ptr());
            }

            wrefresh(window);
        }
    }
}

#[derive(Debug, Clone)]
struct Game {
    width: i16,
    height: i16,
    snake: Snake,
    apples: Apples
}

impl Game {
    fn is_over(&self) -> bool {
        self.snake.cells.iter().any(|&it| it.x < 0 || it.x >= self.width || it.y < 0 || it.y >= self.height) ||
        self.snake.tail().contains(self.snake.head())
    }

    fn score(&self) -> u8 {
        self.snake.cells.len() as u8
    }

    fn update(&self, direction: Option<Direction>) -> Game {
        if self.is_over() {
            return self.clone();
        }

        let (new_snake, new_apples) = self.snake
            .turn(direction)
            .slide()
            .eat(self.apples.to_owned().grow().to_owned()); // to_owned() twice :(

        return Game {
            width: self.width,
            height: self.height,
            snake: new_snake,
            apples: new_apples
        };
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Snake {
    cells: Vec<Cell>,
    direction: Direction,
    eaten_apples: i16
}

impl Snake {
    fn new(cells: Vec<Cell>, direction: Direction) -> Snake {
        Snake { cells, direction, eaten_apples: 0 }
    }

    fn head(&self) -> &Cell {
        &self.cells[0]
    }

    fn tail(&self) -> Vec<Cell> {
        self.cells[1..self.cells.len()].to_owned()
    }

    fn turn(&self, new_direction: Option<Direction>) -> Snake {
        if new_direction.is_none() || are_opposite(new_direction.unwrap(), self.direction) {
            self.clone()
        } else {
            Snake { cells: self.cells.clone(), direction: new_direction.unwrap(), eaten_apples: self.eaten_apples }
        }
    }

    fn slide(&self) -> Snake {
        let mut new_cells = self.cells.clone();

        let new_head = new_cells.first().unwrap().move_in(&self.direction);
        new_cells.insert(0, new_head);

        if self.eaten_apples == 0 {
            new_cells.pop();
        }

        Snake {
            cells: new_cells,
            direction: self.direction,
            eaten_apples: std::cmp::max(0, self.eaten_apples - 1)
        }
    }

    fn eat(&self, apples: Apples) -> (Snake, Apples) {
        if !apples.cells.contains(self.head()) {
            (self.clone(), apples.clone())
        } else {
            let new_snake = Snake {
                cells: self.cells.clone(),
                direction: self.direction,
                eaten_apples: self.eaten_apples + 1
            };

            let mut new_apple_cells = apples.cells.clone();
            new_apple_cells.retain(|it| it != self.head());
            let new_apples = apples.with_cells(new_apple_cells);

            (new_snake, new_apples)
        }
    }
}

#[derive(Debug, Clone)]
struct Apples {
    field_width: i16,
    field_height: i16,
    cells: HashSet<Cell>,
    growth_speed: i16,
    rng: ChaChaRng
}

impl Apples {
    fn create(field_width: i16, field_height: i16) -> Apples {
        let apples = Apples {
            field_width,
            field_height,
            cells: hashset![],
            growth_speed: 3,
            rng: ChaChaRng::from_rng(EntropyRng::new()).unwrap()
        };
        return apples;
    }

    fn with_cells(self, cells: HashSet<Cell>) -> Apples {
        Apples { cells: cells, ..self }
    }

    fn grow(&mut self) -> &mut Apples {
        let n = self.rng.gen_range(0, self.growth_speed);
        if n != 0 {
            return self
        }
        let cell = Cell {
            x: self.rng.gen_range(0, self.field_width),
            y: self.rng.gen_range(0, self.field_height)
        };
        self.cells.insert(cell);

        return self;
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
struct Cell {
    x: i16,
    y: i16,
}

fn cell(x: i16, y: i16) -> Cell {
    return Cell { x, y };
}

impl Cell {
    fn move_in(&self, direction: &Direction) -> Cell {
        let dx = match direction {
            Up => 0,
            Down => 0,
            Left => -1,
            Right => 1,
        };
        let dy = match direction {
            Up => -1,
            Down => 1,
            Left => 0,
            Right => 0,
        };
        return Cell { x: self.x + dx, y: self.y + dy };
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn are_opposite(d1: Direction, d2: Direction) -> bool {
    if d1 == Up && d2 == Down { return true; }
    if d2 == Up && d1 == Down { return true; }
    if d1 == Left && d2 == Right { return true; }
    if d2 == Left && d1 == Right { return true; }
    false
}

trait ToCStr {
    fn to_c_str(&self) -> CString;
}

impl<'a> ToCStr for &'a str {
    fn to_c_str(&self) -> CString {
        CString::new(*self).unwrap()
    }
}

#[link(name = "ncurses")]
#[allow(dead_code)]
extern "C" {
    fn getpid() -> i32;
    fn initscr() -> *mut i8;
    fn noecho() -> i32;
    fn curs_set(c: i8) -> i32;
    fn halfdelay(tenths: i8) -> i32;
    #[link_name = "box"]
    fn box_(window: *mut i8, verch: i8, horch: i8);
    fn mvprintw(y: i16, x: i16, s: *const libc::c_char);
    fn getch() -> i8;
    fn endwin() -> libc::c_int;

    fn newwin(nlines: i16, ncols: i16, begin_y: i8, begin_x: i8) -> *mut i8;
    fn wclear(window: *mut i8) -> i8;
    fn wrefresh(window: *mut i8) -> i8;
    fn mvwprintw(window: *mut i8, y: i16, x: i16, s: *const libc::c_char);
    fn wgetch(window: *mut i8) -> i8;
    fn delwin(window: *mut i8) -> i8;
}


#[cfg(test)]
mod snake_tests {
    use super::*;

    fn new_snake() -> Snake {
        return Snake::new(
            vec![cell(2, 0), cell(1, 0), cell(0, 0)],
            Right,
        );
    }

    #[test]
    fn snake_moves_right() {
        let snake = new_snake();

        assert_eq!(
            snake.slide(),
            Snake::new(
                vec![cell(3, 0), cell(2, 0), cell(1, 0)],
                Right,
            )
        )
    }

    #[test]
    fn snake_can_change_direction() {
        let snake = new_snake();

        assert_eq!(
            snake.turn(Some(Down)).slide(),
            Snake::new(
                vec![cell(2, 1), cell(2, 0), cell(1, 0)],
                Down
            )
        );
        assert_eq!(
            snake.turn(Some(Left)).slide(),
            Snake::new(
                vec![cell(3, 0), cell(2, 0), cell(1, 0)],
                Right
            )
        );
    }

    #[test]
    fn snake_eats_an_apple() {
        let snake = new_snake();
        let apples = Apples::create(10, 10).with_cells(hashset![cell(2, 0)]);

        let (new_snake, new_apples) = snake.eat(apples);
        assert_eq!(new_apples.cells, hashset![]);
        assert_eq!(new_snake.eaten_apples, 1);
        assert_eq!(
            new_snake.slide(),
            Snake {
                cells: vec![cell(3, 0), cell(2, 0), cell(1, 0), cell(0, 0)],
                direction: Right,
                eaten_apples: 0
            }
        );
    }
}

#[cfg(test)]
mod game_tests {
    use super::*;

    #[test]
    fn game_is_over_when_snake_hits_border() {
        let snake = Snake::new(
            vec![cell(2, 0), cell(1, 0), cell(0, 0)],
            Right
        );
        let game = Game { snake, width: 3, height: 1, apples: Apples::create(3, 1) };

        assert_eq!(game.is_over(), false);
        assert_eq!(game.update(None).is_over(), true);
        assert_eq!(game.update(Some(Down)).is_over(), true);
    }

    #[test]
    fn game_is_over_when_snake_bites_itself() {
        let snake = Snake::new(
            vec![cell(0, 0), cell(0, 1), cell(1, 1), cell(1, 0), cell(0, 0)],
            Right
        );
        let game = Game { snake, width: 100, height: 100, apples: Apples::create(3, 1) };

        assert_eq!(game.is_over(), true);
    }
}

#[cfg(test)]
mod apples_tests {
    use super::*;

    #[test]
    fn apples_grow_at_random_locations() {
        let seed = [
            1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5,
            1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2
        ];
        let rng = ChaChaRng::from_seed(seed);
        let mut apples = Apples {
            field_width: 20,
            field_height: 10,
            cells: hashset![],
            growth_speed: 3,
            rng
        };

        assert_eq!(
            apples.grow().grow().grow().cells,
            hashset![cell(7, 0)]
        );
    }
}