use cortex_m::delay::Delay;
use embedded_graphics::Drawable;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{Point, Size, Primitive};
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle, Line};
use embedded_graphics::text::Text;

use crate::games::snake::SnakeGame;
use crate::inputs::Inputs;
use crate::pcd8544::PCD8544;

const WIDTH: usize = 84;

const OPTION_COUNT: usize = 2;
struct Selected(usize);
impl Selected {
    pub fn inc(&mut self) {
        self.0 += 1;
        if self.0 == OPTION_COUNT {
            self.0 = 0;
        }
    }

    pub fn dec(&mut self) {
        if self.0 == 0 {
            self.0 = OPTION_COUNT-1;
        } else {
            self.0 -= 1;
        }
    }
}

pub struct Menu<'a> {
    pcd:    &'a mut PCD8544,
    inputs: &'a mut Inputs,
    delay:  &'a mut Delay,
    selected: Selected,
}

impl<'a> Menu<'a> {
    pub fn new(
        pcd:    &'a mut PCD8544,
        inputs: &'a mut Inputs,
        delay:  &'a mut Delay
        ) -> Self {
        Self{
            pcd,
            inputs,
            delay,
            selected: Selected(0),
        }
    }

    pub fn run(&mut self) {
        self.draw();
        loop {
            self.inputs.update();
            let inputs = self.inputs.is_pressed();
            if inputs[0] {
                match self.selected {
                    Selected(0) => {
                        let mut snake_game = SnakeGame::new(self.pcd, self.inputs, self.delay);
                        snake_game.run();
                        self.draw();
                    },
                    _ => {
                        return;
                    }
                }
            }
            else if inputs[1] {
                self.selected.inc();
                self.draw();
            } else if inputs[2] {
                self.selected.dec();
                self.draw();
            }
            self.delay.delay_ms(100);
        }
    }

    pub fn draw(&mut self) {
        self.pcd.clear();

        Rectangle::new(Point::new(0, 0), Size::new(84, 48))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(self.pcd).unwrap();

        let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        draw_option(self.pcd, style, "rp2040", 8, false);

        Line::new(Point::new(0,12), Point::new(WIDTH as i32, 12))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(self.pcd).unwrap();

        draw_option(self.pcd, style, "Graj", 24, self.selected.0 == 0);
        draw_option(self.pcd, style, "Wyjdz", 38, self.selected.0 == 1);
    }
}

fn draw_option(pcd: &mut PCD8544, style: MonoTextStyle<BinaryColor>, text: &str, y: usize, outline: bool) {
    let pos = center_text(text.len() * 6);
    let font_width = style.font.character_size;
    Text::new(text, Point::new(pos as i32, y as i32), style).draw(pcd).unwrap();
    if outline {
        Rectangle::new(
            Point::new((pos - 2) as i32, y as i32 - font_width.height as i32 +2),
            Size::new((text.len() as u32 * font_width.width) + 3, font_width.height+3))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(pcd).unwrap();
    }
}

fn center_text(width: usize) -> usize {
    (WIDTH - width)/2
}
