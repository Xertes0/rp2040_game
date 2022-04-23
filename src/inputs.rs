use embedded_hal::digital::v2::InputPin;
use rp_pico::hal::gpio::{bank0::{Gpio21, Gpio20, Gpio19, Gpio18}, PullDownInput};
use rp_pico::hal::gpio::Pin;

type GpioMode = PullDownInput;
type GpioIn0 = Gpio21;
type GpioIn1 = Gpio20;
type GpioIn2 = Gpio19;
type GpioIn3 = Gpio18;

pub struct Inputs {
    but0: Pin<GpioIn0, GpioMode>,
    but1: Pin<GpioIn1, GpioMode>,
    but2: Pin<GpioIn2, GpioMode>,
    but3: Pin<GpioIn3, GpioMode>,
    pressed: [bool; 4],
    pressed_once: [bool; 4],
}

impl Inputs {
    pub fn new(
        but0: Pin<GpioIn0, GpioMode>,
        but1: Pin<GpioIn1, GpioMode>,
        but2: Pin<GpioIn2, GpioMode>,
        but3: Pin<GpioIn3, GpioMode>,
    ) -> Self {
        Self {
            but0, but1, but2, but3,
            pressed: [false; 4],
            pressed_once: [false; 4],
        }
    }

    fn update_button(&mut self, is_pressed: bool, index: usize) {
        if is_pressed {
            if !self.pressed[index] {
                self.pressed_once[index] = true;
            } else {
                self.pressed_once[index] = false;
            }
            self.pressed[index] = true;
        } else {
            self.pressed[index] = false;
            self.pressed_once[index] = false;
        }
    }

    pub fn update(&mut self) {
        self.update_button(self.but0.is_high().unwrap(), 0);
        self.update_button(self.but1.is_high().unwrap(), 1);
        self.update_button(self.but2.is_high().unwrap(), 2);
        self.update_button(self.but3.is_high().unwrap(), 3);
    }

    pub fn is_pressed(&self) -> &[bool; 4] {
        &self.pressed_once
    }
}
