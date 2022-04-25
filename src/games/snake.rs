use cortex_m::delay::Delay;
use embedded_graphics::Drawable;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::iso_8859_14::FONT_5X7;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{Point, Size, Primitive};
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle};
use embedded_graphics::text::Text;
use heapless::{String, Vec};

use crate::inputs::Inputs;
use crate::pcd8544::PCD8544;
use crate::rand::Rand;
use crate::sfx::inverse_blink::inverse_blink;

const MAX_SIZE: usize = 100;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Left,
    Down,
    Up,
    Right,
}

const SCALE:  usize = 2;
const GRID_X: usize = 84/SCALE;
const GRID_Y: usize = 48/SCALE;

#[derive(Clone, Copy, Debug)]
struct Tile {
    pub pos: Point,
    pub dir: Direction,
}

struct Snake {
    tail: Vec<Tile, MAX_SIZE>,
    stale: Option<Tile>,
}

impl Snake {
    pub fn new() -> Self {
        Self {
            tail: Vec::from_slice(
              &[ Tile{
                    pos: Point::new(GRID_X as i32/2 - 1, GRID_Y as i32/2),
                    dir: Direction::Right,
                  }
              ]).unwrap(),
            stale: None,
        }
    }

    pub fn set_dir(&mut self, dir: Direction) {
        self.tail[0].dir = dir;
    }

    pub fn draw(&self, pcd: &mut PCD8544) {
        for tile in &self.tail {
            Rectangle::new(tile.pos * SCALE as i32, Size::new(SCALE as u32, SCALE as u32))
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(pcd).unwrap();
        }
    }

    pub fn update(&mut self) -> bool {
        let mut next_dir = self.tail[0].dir;
        for i in 0..self.tail.len() {
            match self.tail[i].dir {
                Direction::Left => {
                    if self.tail[i].pos.x == 1 {
                        return false;
                    }
                    self.tail[i].pos.x -= 1;
                },
                Direction::Up => {
                    if self.tail[i].pos.y == 1 {
                        return false;
                    }
                    self.tail[i].pos.y -= 1;
                },
                Direction::Down => {
                    if self.tail[i].pos.y == GRID_Y as i32 - 2 {
                        return false;
                    }
                    self.tail[i].pos.y += 1;
                },
                Direction::Right => {
                    if self.tail[i].pos.x == GRID_X as i32 - 2 {
                        return false;
                    }
                    self.tail[i].pos.x += 1;
                }
            }

            if i == 0 {
                for other in self.tail.iter().skip(1) {
                    if other.pos.x == self.tail[i].pos.x &&
                       other.pos.y == self.tail[i].pos.y {
                           return false;
                    }
                }
            }
            let tmp = self.tail[i].dir;
            self.tail[i].dir = next_dir;
            next_dir = tmp;
        }

        if let Some(mut stale) = self.stale { // assume stale can't be > 1
            stale.dir = next_dir;
            self.tail.push(stale).unwrap();
            self.stale = None;
        }

        true
    }

    pub fn eat_apple(&mut self, apple: Point) -> bool {
        if self.tail[0].pos == apple {
            self.stale = Some(self.tail[self.tail.len() - 1]);
            return true;
        }

        false
    }

    pub fn get_dir(&self) -> Direction {
        self.tail[0].dir
    }
}

struct Apple {
    pub pos: Point,
    tick: u8,
}

impl Apple {
    pub fn new() -> Self {
        Self {
            pos: Point::new(0,0),
            tick: 0,
        }
    }

    pub fn draw(&mut self, pcd: &mut PCD8544) {
        if self.tick == 2 {
            Rectangle::new(self.pos * SCALE as i32, Size::new(SCALE as u32, SCALE as u32))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(pcd).unwrap();
            self.tick = 0;
        } else {
            self.tick += 1;
        }
    }
}

pub struct SnakeGame<'a> {
    pcd:    &'a mut PCD8544,
    inputs: &'a mut Inputs,
    delay:  &'a mut Delay,
    snake: Snake,
    apple: Apple,
    rand:  Rand,
}

impl<'a> SnakeGame<'a> {
    pub fn new(
        pcd:    &'a mut PCD8544,
        inputs: &'a mut Inputs,
        delay:  &'a mut Delay
        ) -> Self {
        Self{
            pcd,
            inputs,
            delay,
            snake: Snake::new(),
            apple: Apple::new(),
            rand:  Rand::new(25023540),
        }
    }

    fn make_apple(&mut self) {
        self.apple.pos.x = ((self.rand.next() as usize % (GRID_X-5)) + 2) as i32;
        self.apple.pos.y = ((self.rand.next() as usize % (GRID_Y-5)) + 2) as i32;
        if self.apple.pos.x % 2 == 1 {
            self.apple.pos.x += 1;
        }

        if self.apple.pos.y % 2 == 1 {
            self.apple.pos.y += 1;
        }
    }

    pub fn run(&mut self) {
        self.pcd.clear();
        self.make_apple();
        loop {
            if !self.update() {
                return;
            }
            self.draw();

            self.delay.delay_ms(300);
        }
    }

    fn update(&mut self) -> bool {
        self.inputs.update();
        let inputs = self.inputs.is_pressed();
        if inputs[0] && self.snake.get_dir() != Direction::Right {
            self.snake.set_dir(Direction::Left);
        } else if inputs[1] && self.snake.get_dir() != Direction::Up {
            self.snake.set_dir(Direction::Down);
        } else if inputs[2] && self.snake.get_dir() != Direction::Down {
            self.snake.set_dir(Direction::Up);
        } else if inputs[3] && self.snake.get_dir() != Direction::Left {
            self.snake.set_dir(Direction::Right);
        }

        if self.snake.eat_apple(self.apple.pos) {
            inverse_blink(self.pcd, self.delay, 500, 1);
            self.make_apple();
        }

        if !self.snake.update() {
            self.pcd.inverse();
            self.pcd.draw();
            loop {
                self.inputs.update();
                match self.inputs.is_pressed().iter().find(|x| **x) {
                    Some(_) => { return false; },
                    None => {}
                }

                self.delay.delay_ms(20);
            }
        }

        true
    }

    fn draw(&mut self) {
        // Clear
        self.pcd.clear();

        // Border
        Rectangle::new(Point::new(0,0), Size::new(84, 48))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(self.pcd).unwrap();

        // Snake
        self.snake.draw(self.pcd);

        // Apple
        self.apple.draw(self.pcd);

        // Score
        let x_size = if self.snake.tail.len() > 9 {
            10
        } else {
            5
        };

        Rectangle::new(Point::new(0,45-7), Size::new(x_size+3, 7+3))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(self.pcd).unwrap();
        Text::new(&String::<3>::from(self.snake.tail.len() as u32), Point::new(2,45), MonoTextStyle::new(&FONT_5X7, BinaryColor::On))
            .draw(self.pcd).unwrap();

        self.pcd.draw();
    }
}
