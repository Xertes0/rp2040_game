use cortex_m::delay::Delay;
use embedded_graphics::Drawable;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{Point, Size, Primitive};
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle};

use crate::inputs::Inputs;
use crate::pcd8544::PCD8544;

const BRICK_X_COUNT: usize = 5;
const BRICK_Y_COUNT: usize = 4;
//const BRICK_COUNT: usize = BRICK_X_COUNT * BRICK_Y_COUNT;
const BRICK_X_SPACE: usize = 2;
const BRICK_SIZE: Size = Size::new(13, 3);
const BRICK_X_OFFSET: usize = (84 - (BRICK_X_COUNT * BRICK_SIZE.width as usize + ((BRICK_X_COUNT-1) * BRICK_X_SPACE))) / 2;
const BRICK_Y_OFFSET: usize = 3;

const BALL_SIZE: Size = Size::new(2,2);

#[derive(Clone, Copy)]
struct Brick {
    active: bool,
}

impl Brick {
    pub fn check_collision(pos: Point, ball: Point) -> bool {
        if  ball.x >= pos.x &&
            ball.y >= pos.y &&
            ball.x + BALL_SIZE.width as i32  <= pos.x + BRICK_SIZE.width as i32 &&
            ball.y + BALL_SIZE.height as i32 <= pos.y + BRICK_SIZE.height as i32
        {
            true
        } else {
            false
        }
    }

    pub fn actual_pos(x: usize, y: usize) -> Point {
        Point::new(
            (x as i32 * (BRICK_SIZE.width as i32 + BRICK_X_SPACE as i32)) + BRICK_X_OFFSET as i32,
            (y as i32 * 5) + BRICK_Y_OFFSET as i32
        )
    }
}

struct Ball {
    pos: Point,
    vel: (i32, i32),
}

impl Ball {
    pub fn new() -> Self {
        Self {
            pos: Point::new(84/2, 45),
            vel: (2,-1),
        }
    }

    pub fn update(&mut self) {
        if self.pos.x >= 84 - BALL_SIZE.width as i32 {
            self.reflect_x();
        }
        if self.pos.x <= 1 {
            self.reflect_x();
        }
        if self.pos.y >= 48 - BALL_SIZE.height as i32 {
            self.reflect_y();
        }
        if self.pos.y <= 1 {
            self.reflect_y();
        }

        self.pos.x += self.vel.0;
        self.pos.y += self.vel.1;
    }

    pub fn reflect_x(&mut self) {
        self.vel.0 *= -1;
    }

    pub fn reflect_y(&mut self) {
        self.vel.1 *= -1;
    }

    pub fn draw(&self, pcd: &mut PCD8544) {
        Rectangle::new(self.pos, BALL_SIZE)
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(pcd).unwrap();
    }
}

pub struct Breakout<'a> {
    pcd:    &'a mut PCD8544,
    inputs: &'a mut Inputs,
    delay:  &'a mut Delay,
    bricks: [[Brick; BRICK_Y_COUNT]; BRICK_X_COUNT],
    ball: Ball,
}

impl<'a> Breakout<'a> {
    pub fn new(
        pcd:    &'a mut PCD8544,
        inputs: &'a mut Inputs,
        delay:  &'a mut Delay,
    ) -> Self {
        Self {
            pcd, inputs, delay,
            bricks: [[Brick{active:true}; BRICK_Y_COUNT]; BRICK_X_COUNT],
            ball: Ball::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            if !self.update() {
                return;
            }

            self.draw();
            self.delay.delay_ms(150);
        }
    }

    fn update(&mut self) -> bool {
        self.ball.update();
        for x in 0..BRICK_X_COUNT {
            for y in 0..BRICK_Y_COUNT {
                if !self.bricks[x][y].active { continue; }
                let pos = Brick::actual_pos(x,y);
                if Brick::check_collision(pos, self.ball.pos) {
                    self.bricks[x][y].active = false;

                    let x = (pos.x + (BRICK_SIZE.width as i32 / 2)) - (self.ball.pos.x + (BALL_SIZE.width as i32 / 2));
                    let y = (pos.y + (BRICK_SIZE.height as i32 / 2)) - (self.ball.pos.y + (BALL_SIZE.height as i32 / 2));
                    if x.abs() > y.abs()
                    {
                        self.ball.reflect_x();
                    }
                    else
                    {
                        self.ball.reflect_y();
                    }
                }
            }
        }
        true
    }

    fn draw(&mut self) {
        self.pcd.clear();

        // Border
        Rectangle::new(Point::new(0, 0), Size::new(84, 48))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(self.pcd).unwrap();

        // Bricks
        for x in 0..BRICK_X_COUNT {
            for y in 0..BRICK_Y_COUNT {
                if !self.bricks[x][y].active { continue; }

                Rectangle::new(Brick::actual_pos(x,y), BRICK_SIZE)
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(self.pcd).unwrap();
            }
        }

        // Ball
        self.ball.draw(self.pcd);

        self.pcd.draw();
    }
}
