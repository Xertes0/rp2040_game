use cortex_m::delay::Delay;
use embedded_graphics::Drawable;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{Point, Size, Primitive};
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle};

use crate::inputs::Inputs;
use crate::pcd8544::PCD8544;
use crate::rand::Rand;

const MAX_SIZE: usize = 30;

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Down,
    Top,
    Right,
}

const SCALE:  usize = 2;
const GRID_X: usize = 84/SCALE;
const GRID_Y: usize = 48/SCALE;

#[derive(Clone, Copy)]
struct Tile {
    pub pos: Point,
    pub dir: Direction,
}

struct Snake {
    tail: [Tile; MAX_SIZE],
    tail_count: usize,
    stale: usize,
}

impl Snake {
    pub fn new() -> Self {
        Self {
            tail: [Tile{ pos: Point::new(GRID_X as i32/2 - 1, GRID_Y as i32/2), dir: Direction::Right }; MAX_SIZE],
            tail_count: 1,
            stale: 0,
        }
    }

    pub fn set_dir(&mut self, dir: Direction) {
        self.tail[0].dir = dir;
    }

    pub fn draw(&self, pcd: &mut PCD8544) {
        for i in 0..self.tail_count {
            Rectangle::new(self.tail[i].pos * SCALE as i32, Size::new(SCALE as u32, SCALE as u32))
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(pcd).unwrap();
        }
    }

    fn check_collision(&self, index: usize) -> bool {
        for i in 0..self.tail_count-self.stale {
            if i == index {continue;}
            if self.tail[i].pos.x == self.tail[index].pos.x &&
               self.tail[i].pos.y == self.tail[index].pos.y {
                   return true;
            }
        }

        false
    }

    pub fn update(&mut self) -> bool {
        let mut next_dir = self.tail[0].dir;
        for i in 0..self.tail_count-self.stale {

            match self.tail[i].dir {
                Direction::Left => {
                    if self.tail[i].pos.x == 1 {
                        return false;
                    }
                    self.tail[i].pos.x -= 1;
                    if self.check_collision(i) {
                        return false;
                    }
                },
                Direction::Top => {
                    if self.tail[i].pos.y == 1 {
                        return false;
                    }
                    self.tail[i].pos.y -= 1;
                    if self.check_collision(i) {
                        return false;
                    }
                },
                Direction::Down => {
                    if self.tail[i].pos.y == GRID_Y as i32 - 2 {
                        return false;
                    }
                    self.tail[i].pos.y += 1;
                    if self.check_collision(i) {
                        return false;
                    }
                },
                Direction::Right => {
                    if self.tail[i].pos.x == GRID_X as i32 - 2 {
                        return false;
                    }
                    self.tail[i].pos.x += 1;
                    if self.check_collision(i) {
                        return false;
                    }
                }
            }
            let tmp = self.tail[i].dir;
            self.tail[i].dir = next_dir;
            next_dir = tmp;
        }

        if self.stale > 0 { // assume stale can't be > 1
            self.stale -= 1;
            self.tail[self.tail_count].dir = next_dir;
        }

        true
    }

    pub fn eat_apple(&mut self, apple: Point) -> bool {
        if self.tail[0].pos == apple {
            self.tail[self.tail_count] = self.tail[self.tail_count-1];
            self.tail_count += 1;
            self.stale += 1;
            return true;
        }

        false
    }
}

pub struct SnakeGame<'a> {
    pcd:    &'a mut PCD8544,
    inputs: &'a mut Inputs,
    delay:  &'a mut Delay,
    snake: Snake,
    apple: Point,
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
            apple: Point::new(0, 0),
            rand:  Rand::new(25023540),
        }
    }

    fn make_apple(&mut self) {
        self.apple.x = ((self.rand.next() as usize % (GRID_X-5)) + 2) as i32;
        self.apple.y = ((self.rand.next() as usize % (GRID_Y-5)) + 2) as i32;
        if self.apple.x % 2 == 1 {
            self.apple.x += 1;
        }

        if self.apple.y % 2 == 1 {
            self.apple.y += 1;
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
        if inputs[0] {
            self.snake.set_dir(Direction::Left);
        } else if inputs[1] {
            self.snake.set_dir(Direction::Down);
        } else if inputs[2] {
            self.snake.set_dir(Direction::Top);
        } else if inputs[3] {
            self.snake.set_dir(Direction::Right);
        }

        if self.snake.eat_apple(self.apple) {
            self.make_apple();
        }

        if !self.snake.update() {
            return false;
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
        Rectangle::new(self.apple * SCALE as i32, Size::new(SCALE as u32, SCALE as u32))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(self.pcd).unwrap();
    }
}
