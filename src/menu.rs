use cortex_m::delay::Delay;
use embedded_graphics::Drawable;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{Point, Size, Primitive};
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle, Line};
use embedded_graphics::text::Text;

use crate::inputs::Inputs;
use crate::pcd8544::PCD8544;

const WIDTH: usize = 84;

struct Selected<const OPTION_COUNT: usize>(usize);
impl<const OPTION_COUNT: usize> Selected<OPTION_COUNT> {
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

    pub fn peek(&self) -> usize {
        if self.0 + 1 == OPTION_COUNT {
            0
        } else {
            self.0 + 1
        }
    }
}

pub struct MenuOption<'a, OptionId: Copy>{
    id: OptionId,
    text: &'a str,
}

impl<'a, OptionId: Copy> MenuOption<'a, OptionId> {
    pub fn new(id: OptionId, text: &'a str) -> Self {
        Self {
            id,
            text,
        }
    }
}

pub struct Menu<'a, OptionId: Copy, const OPTION_COUNT: usize> {
    header: &'a str,
    options: [MenuOption<'a, OptionId>; OPTION_COUNT],
    selected: Selected<OPTION_COUNT>,
}

impl<'a, OptionId: Copy, const OPTION_COUNT: usize> Menu<'a, OptionId, OPTION_COUNT> {
    pub fn new(header: &'a str, options: [MenuOption<'a, OptionId>; OPTION_COUNT]) -> Self {
        Self{
            header,
            options,
            selected: Selected::<OPTION_COUNT>(0),
        }
    }

    pub fn run(&mut self, pcd: &mut PCD8544, inputs: &mut Inputs, delay: &mut Delay) -> OptionId {
        self.draw(pcd);
        loop {
            inputs.update();
            let inputs = inputs.is_pressed();
            if inputs[0] {
                return self.options[self.selected.0].id;
            }
            else if inputs[1] {
                self.selected.inc();
                self.draw(pcd);
            } else if inputs[2] {
                self.selected.dec();
                self.draw(pcd);
            }
            delay.delay_ms(100);
        }
    }

    pub fn draw(&self, pcd: &mut PCD8544) {
        pcd.clear();

        // Border
        Rectangle::new(Point::new(0, 0), Size::new(84, 48))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(pcd).unwrap();

        // Text style
        let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

        // Header
        self.draw_header(pcd, self.header, style);
        Line::new(Point::new(0,11), Point::new(WIDTH as i32, 11))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(pcd).unwrap();

        // Options
        self.draw_options(pcd, style);

        // Draw to pcd
        pcd.draw();
    }

    fn draw_header(&self, pcd: &mut PCD8544, text: &str, style: MonoTextStyle<BinaryColor>) {
        let pos = center_text(text.len() * 6);
        Text::new(text, Point::new(pos as i32, 8), style).draw(pcd).unwrap();
    }

    fn draw_options(&self, pcd: &mut PCD8544, style: MonoTextStyle<BinaryColor>) {
        let mut first = true;
        for i in [self.selected.0, self.selected.peek()] {
            let y = if first {
                first = false;
                24
            } else {
                38
            };

            let text = self.options[i].text;
            let pos = center_text(text.len() * 6);
            let font_width = style.font.character_size;
            Text::new(text, Point::new(pos as i32, y as i32), style).draw(pcd).unwrap();
            if self.selected.0 == i {
                Rectangle::new(
                    Point::new((pos - 3) as i32, y as i32 - font_width.height as i32 + 2),
                    Size::new((text.len() as u32 * font_width.width) + 5, font_width.height+3))
                    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                    .draw(pcd).unwrap();
            }
        }
    }
}

fn center_text(width: usize) -> usize {
    (WIDTH - width)/2
}
