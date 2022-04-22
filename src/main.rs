//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use cortex_m::{delay::Delay, prelude::_embedded_hal_blocking_spi_Write};
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
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

const FUNCTION_SET: u8 = 0x20;
const ADDRESSING_VERT: u8 = 0x02;
const EXTENDED_INSTR: u8 = 0x01;
const TEMP_COEFF_2: u8 = 0x06;
const BIAS_1_40: u8 = 0x14;
const SET_VOP: u8 = 0x80;
const DISPLAY_NORMAL: u8 = 0x0c;

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
    let mut spi = Spi::<_, _, 8>::new(pac.SPI0).init(&mut pac.RESETS, 125_000_000u32.Hz(), 2_000_000u32.Hz(), &MODE_0);

    ce_pin.set_high().unwrap();
    dc_pin.set_low().unwrap();

    rst_pin.set_high().unwrap();
    delay.delay_us(100);
    rst_pin.set_low().unwrap();
    delay.delay_us(100);
    rst_pin.set_high().unwrap();
    delay.delay_us(100);

    led_pin.set_low().unwrap();

    let fnset = FUNCTION_SET & !ADDRESSING_VERT;
    command(&mut dc_pin, &mut ce_pin, &mut spi, fnset);
    command(&mut dc_pin, &mut ce_pin, &mut spi, fnset | EXTENDED_INSTR);
    command(&mut dc_pin, &mut ce_pin, &mut spi, TEMP_COEFF_2);
    command(&mut dc_pin, &mut ce_pin, &mut spi, BIAS_1_40);
    command(&mut dc_pin, &mut ce_pin, &mut spi, SET_VOP | 0x3f);
    command(&mut dc_pin, &mut ce_pin, &mut spi, fnset & !EXTENDED_INSTR);
    command(&mut dc_pin, &mut ce_pin, &mut spi, DISPLAY_NORMAL);

    //clear
    draw(&mut dc_pin, &mut ce_pin, &mut spi, &[200u8;84*48/8]);

    loop {}

    //lcd_command(&mut din_pin, &mut clk_pin, &mut delay, 0x20 | 0x01);
    //lcd_command(&mut din_pin, &mut clk_pin, &mut delay, 0x13 | 0x03);
    //lcd_command(&mut din_pin, &mut clk_pin, &mut delay, 0x80 | 0x32);
    //lcd_command(&mut din_pin, &mut clk_pin, &mut delay, 0x20);
    //lcd_command(&mut din_pin, &mut clk_pin, &mut delay, 0x08 | 0x04);

    //delay.delay_ms(500);

    //led_pin.set_high().unwrap();
    //// draw

    //let mut buffer = [true; 48*84];
    ////let mut is_on = false;
    ////for i in 0..1 {
    //    lcd_draw(&mut dc_pin, &mut din_pin, &mut clk_pin, &mut led_pin, &mut delay, &buffer);
    //    led_pin.set_low().unwrap();
    //    //buffer[i * 10] = false;
    //    //delay.delay_ms(100);
    //    //if is_on {
    //    //    led_pin.set_low().unwrap();
    //    //    is_on = false;
    //    //} else {
    //    //    led_pin.set_high().unwrap();
    //    //    is_on = true;
    //    //}
    ////}

    //dc_pin.set_low().unwrap();
    //rst_pin.set_low().unwrap();
    //clk_pin.set_low().unwrap();
    //din_pin.set_low().unwrap();
    //ce_pin.set_low().unwrap();

    //led_pin.set_low().unwrap();

    //loop {
    //}
}

fn command<GPIO0: PinId, GPIO1: PinId, A: bsp::hal::spi::SpiDevice>(
    dc: &mut Pin<GPIO0, Output<PushPull>>,
    ce: &mut Pin<GPIO1, Output<PushPull>>,
    spi: &mut Spi<bsp::hal::spi::Enabled, A, 8>,
    data: u8
) {
    dc.set_low().unwrap();
    ce.set_low().unwrap();

    spi.write(&[data]).unwrap();

    ce.set_high().unwrap();
}

fn draw<GPIO0: PinId, GPIO1: PinId, A: bsp::hal::spi::SpiDevice>(
    dc: &mut Pin<GPIO0, Output<PushPull>>,
    ce: &mut Pin<GPIO1, Output<PushPull>>,
    spi: &mut Spi<bsp::hal::spi::Enabled, A, 8>,
    data: &[u8]
) {
    dc.set_high().unwrap();
    ce.set_low().unwrap();

    spi.write(data).unwrap();

    ce.set_high().unwrap();
}

//const DELAY: u32 = 10;
//
//fn lcd_draw<GPIO0: PinId, GPIO1: PinId, GPIO2: PinId, GPIO3: PinId> (
//    dc_pin:  &mut Pin<GPIO0, Output<PushPull>>,
//    din_pin: &mut Pin<GPIO1, Output<PushPull>>,
//    clk_pin: &mut Pin<GPIO2, Output<PushPull>>,
//    led_pin: &mut Pin<GPIO3, Output<PushPull>>,
//    delay:   &mut Delay,
//    data: &[bool; 48*84])
//{
//    for x in 0u8..84u8 {
//        dc_pin.set_low().unwrap();
//        lcd_command(din_pin, clk_pin, delay, 0x80 | x);
//
//        for y in 0u8..6u8 {
//            dc_pin.set_low().unwrap();
//            lcd_command(din_pin, clk_pin, delay, 0x40 | y);
//
//            dc_pin.set_high().unwrap();
//            for _ in 0u8..8u8 {
//                //if data[((i * 48 * 6) + (y * 48) + x) as usize] {
//                //if i & 2 == 0 {
//                    din_pin.set_high().unwrap();
//                //} else {
//                    //din_pin.set_low().unwrap();
//                //}
//                clk_pin.set_high().unwrap();
//                delay.delay_ms(DELAY);
//                clk_pin.set_low().unwrap();
//            }
//        }
//        led_pin.set_low().unwrap();
//        return ();
//    }
//
//    lcd_command(din_pin, clk_pin, delay, 0x40);
//}

//fn lcd_command<GPIO0: PinId, GPIO1: PinId> (
//    din_pin: &mut Pin<GPIO0, Output<PushPull>>,
//    clk_pin: &mut Pin<GPIO1, Output<PushPull>>,
//    delay:   &mut Delay,
//    data: u8)
//{
//}

// End of file
