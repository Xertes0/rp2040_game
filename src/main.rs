//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

mod pcd8544;
use pcd8544::PCD8544;

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::{Point, Size, Primitive}, primitives::{Circle, PrimitiveStyle, Rectangle}, Drawable, mono_font::{MonoTextStyle, iso_8859_10::FONT_4X6, ascii::FONT_6X10, iso_8859_1::FONT_5X7}, text::Text};
use embedded_hal::{digital::v2::OutputPin, spi::MODE_0};
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
    watchdog::Watchdog, gpio::{Pin, Output, PushPull, PinId, FunctionSpi}, Spi,
};

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

    led_pin.set_high().unwrap();

    let mut rst_pin = pins.gpio8.into_push_pull_output();
    let mut ce_pin  = pins.gpio5.into_push_pull_output();
    let mut dc_pin  = pins.gpio4.into_push_pull_output();
    //let mut din_pin = pins.gpio7.into_push_pull_output();
    //let mut clk_pin = pins.gpio6.into_push_pull_output();
    let _ = pins.gpio6.into_mode::<FunctionSpi>();
    let _ = pins.gpio7.into_mode::<FunctionSpi>();
    let spi = Spi::<_, _, 8>::new(pac.SPI0).init(&mut pac.RESETS, 125_000_000u32.Hz(), 2_000_000u32.Hz(), &MODE_0);

    ce_pin.set_high().unwrap();
    dc_pin.set_low().unwrap();

    rst_pin.set_high().unwrap();
    delay.delay_us(100);
    rst_pin.set_low().unwrap();
    delay.delay_us(100);
    rst_pin.set_high().unwrap();
    delay.delay_us(100);

    led_pin.set_low().unwrap();

    let mut pcd = PCD8544::new(rst_pin, ce_pin, dc_pin, spi);

    //let raw_image = ImageRaw::<BinaryColor>::new(&[0;84*48], 84);
    //let mut image = Image::new(&raw_image, Point::zero());
    //Rectangle::new(Point::new(0, 20), Size::new(10, 10))
    //    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
    //    .draw(&mut pcd).unwrap();
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    //Text::new("Abc!", Point::new(5, 15), style).draw(&mut pcd).unwrap();
    Text::new("Ekran test", Point::new(5, 15), style).draw(&mut pcd).unwrap();

    Circle::new(Point::new(10, 20), 30)
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut pcd).unwrap();

    loop {}
}

// End of file
