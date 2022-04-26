#![no_std]
#![no_main]

mod pcd8544;
mod inputs;
mod menu;
mod games;
mod rand;
mod sfx;

use games::games_menu::GamesMenu;
use inputs::Inputs;
use menu::{Menu, MenuOption};
use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use pcd8544::PCD8544;

use cortex_m_rt::entry;
use embedded_hal::{digital::v2::{OutputPin, ToggleableOutputPin}, spi::MODE_0};
use embedded_time::{fixed_point::FixedPoint, rate::Extensions};

use rp_pico as bsp;
use panic_halt as _;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog, gpio::FunctionSpi, Spi, rom_data::reset_to_usb_boot,
};

use pac::interrupt;

type RebootPin = bsp::hal::gpio::Pin<bsp::hal::gpio::bank0::Gpio22, bsp::hal::gpio::PullDownInput>;
static REBOOT_PIN: Mutex<RefCell<Option<RebootPin>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
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

    let mut vcc_pin = pins.gpio0.into_push_pull_output();
    {
        led_pin.set_high().unwrap();
        let mut bl_pin  = pins.gpio1.into_push_pull_output();
        let rst_pin = pins.gpio8.into_push_pull_output();
        let ce_pin  = pins.gpio5.into_push_pull_output();
        let dc_pin  = pins.gpio4.into_push_pull_output();
        //let mut din_pin = pins.gpio7.into_push_pull_output();
        //let mut clk_pin = pins.gpio6.into_push_pull_output();
        let _ = pins.gpio6.into_mode::<FunctionSpi>();
        let _ = pins.gpio7.into_mode::<FunctionSpi>();
        let spi = Spi::<_, _, 8>::new(pac.SPI0).init(&mut pac.RESETS, 125_000_000u32.Hz(), 2_000_000u32.Hz(), &MODE_0);

        vcc_pin.set_high().unwrap();
        bl_pin.set_high().unwrap();
        let mut pcd = PCD8544::new(rst_pin, ce_pin, dc_pin, spi, &mut delay);

        led_pin.set_low().unwrap();

        let mut inputs = Inputs::new(
            pins.gpio21.into_pull_down_input(),
            pins.gpio20.into_pull_down_input(),
            pins.gpio19.into_pull_down_input(),
            pins.gpio18.into_pull_down_input(),
        );

        #[derive(Clone, Copy)]
        enum MenuSelected {
            Play, Backlight, Quit
        }

        'menu: loop {
            let mut menu = Menu::new([
                MenuOption::new(MenuSelected::Play, "Graj"),
                MenuOption::new(MenuSelected::Backlight, "Podswl"),
                MenuOption::new(MenuSelected::Quit, "Wyjdz"),
            ]);

            match menu.run(&mut pcd, &mut inputs, &mut delay) {
                MenuSelected::Play => {
                    GamesMenu::run(&mut pcd, &mut inputs, &mut delay);
                },
                MenuSelected::Backlight => {
                    bl_pin.toggle().unwrap();
                },
                MenuSelected::Quit => {
                    break 'menu;
                }
            }
        }

        bl_pin.set_low().unwrap();
    }
    vcc_pin.set_low().unwrap();

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
