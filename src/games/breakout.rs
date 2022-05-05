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

#[derive(Clone, Copy)]
struct Brick {
    active: bool,
}

#[derive(Clone, Copy)]
struct Ball {
    pos: Point,
}

impl Ball {
    pub fn new() -> Self {
        Self {
            pos: Point::new(84/2, 8)
        }
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
            self.delay.delay_ms(100);
        }
    }

    fn update(&mut self) -> bool {
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

                Rectangle::new(Point::new(
                        (x as i32 * (BRICK_SIZE.width as i32 + BRICK_X_SPACE as i32)) + BRICK_X_OFFSET as i32,
                        (y as i32 * 5) + BRICK_Y_OFFSET as i32
                        ), BRICK_SIZE)
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(self.pcd).unwrap();
            }
        }

        // Ball

        self.pcd.draw();
    }
}
