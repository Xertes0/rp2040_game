use cortex_m::{prelude::_embedded_hal_blocking_spi_Write, delay::Delay};
use embedded_graphics::{prelude::{OriginDimensions, Size}, pixelcolor::BinaryColor, draw_target::DrawTarget, Pixel};
use embedded_hal::digital::v2::OutputPin;
use rp_pico::hal::{gpio::{Pin, Output, PushPull, bank0::{Gpio8, Gpio5, Gpio4}}, spi::{Spi, Enabled}};
use rp_pico::pac::SPI0;

const FUNCTION_SET: u8 = 0x20;
const ADDRESSING_VERT: u8 = 0x02;
const EXTENDED_INSTR: u8 = 0x01;
const TEMP_COEFF_2: u8 = 0x06;
const BIAS_1_40: u8 = 0x14;
const SET_VOP: u8 = 0x80;
const DISPLAY_NORMAL: u8 = 0x0c;
const POWER_DOWN: u8 = 0x04;

type GpioRst = Gpio8;
type GpioCe  = Gpio5;
type GpioDc  = Gpio4;
type SpiPin  = SPI0;

#[allow(non_camel_case_types)]
pub struct PCD8544 {
    rst: Pin<GpioRst, Output<PushPull>>,
    ce:  Pin<GpioCe, Output<PushPull>>,
    dc:  Pin<GpioDc, Output<PushPull>>,
    spi: Spi<Enabled, SpiPin, 8>,
    pub fnset: u8,
    draw_buffer: [u8; 84*48/8],
}

#[allow(non_camel_case_types)]
impl PCD8544 {
    pub fn new(
            rst: Pin<GpioRst, Output<PushPull>>,
            ce:  Pin<GpioCe, Output<PushPull>>,
            dc:  Pin<GpioDc, Output<PushPull>>,
            spi: Spi<Enabled, SpiPin, 8>,
            delay: &mut Delay
    ) -> Self {
        let mut pcd = Self {
            fnset: FUNCTION_SET & !ADDRESSING_VERT,
            rst,
            ce,
            dc,
            spi,
            draw_buffer: [0; 84*48/8],
        };
        pcd.init(delay);
        pcd
    }

    fn init(&mut self, delay: &mut Delay) {
        self.ce.set_high().unwrap();
        self.dc.set_low().unwrap();

        self.rst.set_high().unwrap();
        delay.delay_us(100);
        self.rst.set_low().unwrap();
        delay.delay_us(100);
        self.rst.set_high().unwrap();
        delay.delay_us(100);

        self.command(self.fnset);
        self.command(self.fnset | EXTENDED_INSTR);
        self.command(TEMP_COEFF_2);
        self.command(BIAS_1_40);
        self.command(SET_VOP | 0x3f);
        self.command(self.fnset & !EXTENDED_INSTR);
        self.command(DISPLAY_NORMAL);

        //clear with 1s
        self.set(255);
    }

    pub fn set(&mut self, value: u8) {
        self.draw(&[value;84*48/8]);
    }

    pub fn clear(&mut self) {
        self.draw_buffer = [0; 84*48/8];
        self.draw_self();
    }

    fn command(&mut self, data: u8) {
        self.dc.set_low().unwrap();
        self.ce.set_low().unwrap();

        self.spi.write(&[data]).unwrap();

        self.ce.set_high().unwrap();
    }

    fn draw(&mut self, data: &[u8]) {
        self.dc.set_high().unwrap();
        self.ce.set_low().unwrap();

        self.spi.write(data).unwrap();

        self.ce.set_high().unwrap();
    }

    fn draw_self(&mut self) {
        self.dc.set_high().unwrap();
        self.ce.set_low().unwrap();

        self.spi.write(&self.draw_buffer).unwrap();

        self.ce.set_high().unwrap();
    }
}

#[allow(non_camel_case_types)]
impl Drop for PCD8544 {
    fn drop(&mut self) {
        self.fnset |= POWER_DOWN;
        self.command(self.fnset);
    }
}

#[allow(non_camel_case_types)]
impl OriginDimensions for PCD8544 {
    fn size(&self) -> Size {
        Size::new(84, 48)
    }
}

#[allow(non_camel_case_types)]
impl DrawTarget for PCD8544 {
    type Color = BinaryColor;

    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>> {
        //self.draw_buffer = [0; 84*48/8];
        for Pixel(coord, color) in pixels.into_iter() {
            //let x = coord.x as usize;
            //let y = (coord.y.abs() / 8) as usize;
            //let i = (coord.y.abs() % 8) as u8;
            let index = ((coord.y >> 3) * 84 + coord.x) as usize;
            let offset = coord.y as usize & 0x07;
            self.draw_buffer[index] = (self.draw_buffer[index] & !(0x01 << offset)) | ((if color.is_on() {1} else {0}) << offset);
        }

        self.draw_self();
        Ok(())
    }
}
