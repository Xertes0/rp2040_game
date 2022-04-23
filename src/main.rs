//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

mod pcd8544;
mod inputs;
mod menu;
use inputs::Inputs;
use menu::Menu;
use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use pcd8544::PCD8544;

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::{Point, Size, Primitive}, primitives::{Circle, PrimitiveStyle, Rectangle}, Drawable, mono_font::{MonoTextStyle, iso_8859_10::FONT_4X6, ascii::FONT_6X10, iso_8859_1::FONT_5X7}, text::Text};
use embedded_hal::{digital::v2::{OutputPin, InputPin, ToggleableOutputPin}, spi::MODE_0};
use embedded_time::{fixed_point::FixedPoint, rate::Extensions};
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog, gpio::{Pin, Output, PushPull, PinId, FunctionSpi}, Spi, rom_data::reset_to_usb_boot,
};

use pac::interrupt;

type RebootPin = bsp::hal::gpio::Pin<bsp::hal::gpio::bank0::Gpio22, bsp::hal::gpio::PullDownInput>;
static REBOOT_PIN: Mutex<RefCell<Option<RebootPin>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();

    let reboot_pin = pins.gpio22.into_pull_down_input();
    reboot_pin.set_interrupt_enabled(bsp::hal::gpio::Interrupt::EdgeHigh, true);
    cortex_m::interrupt::free(|cs| {
        REBOOT_PIN.borrow(cs).replace(Some(reboot_pin));
    });

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
    }

    {
        led_pin.set_high().unwrap();
        let rst_pin = pins.gpio8.into_push_pull_output();
        let ce_pin  = pins.gpio5.into_push_pull_output();
        let dc_pin  = pins.gpio4.into_push_pull_output();
        //let mut din_pin = pins.gpio7.into_push_pull_output();
        //let mut clk_pin = pins.gpio6.into_push_pull_output();
        let _ = pins.gpio6.into_mode::<FunctionSpi>();
        let _ = pins.gpio7.into_mode::<FunctionSpi>();
        let spi = Spi::<_, _, 8>::new(pac.SPI0).init(&mut pac.RESETS, 125_000_000u32.Hz(), 2_000_000u32.Hz(), &MODE_0);

        let mut pcd = PCD8544::new(rst_pin, ce_pin, dc_pin, spi, &mut delay);

        led_pin.set_low().unwrap();

        let mut inputs = Inputs::new(
            pins.gpio21.into_pull_down_input(),
            pins.gpio20.into_pull_down_input(),
            pins.gpio19.into_pull_down_input(),
            pins.gpio18.into_pull_down_input(),
        );

        let mut menu = Menu::new(&mut pcd, &mut inputs, &mut delay);
        menu.run();
    }

    loop {
        led_pin.toggle().unwrap();
        delay.delay_ms(1000);
        cortex_m::asm::nop();
    }
}

#[allow(non_snake_case)]
#[interrupt]
fn IO_IRQ_BANK0() {
    static mut REBOOT_BUTTON: Option<RebootPin> = None;

    if REBOOT_BUTTON.is_none() {
        cortex_m::interrupt::free(|cs| {
            *REBOOT_BUTTON = REBOOT_PIN.borrow(cs).take();
        });
    }

    if let Some(button) = REBOOT_BUTTON {
        reset_to_usb_boot(0, 0);
        button.clear_interrupt(bsp::hal::gpio::Interrupt::EdgeHigh);
    }
}

// End of file
