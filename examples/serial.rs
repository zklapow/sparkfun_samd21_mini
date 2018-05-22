#![feature(used)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate atsamd21_hal as atsamd21;
extern crate sparkfun_samd21_mini as hal;
extern crate embedded_hal;
extern crate panic_abort;

use hal::prelude::*;
use hal::clock::{GenericClockController};
use hal::delay::Delay;
use hal::sercom::{Uart, PadPin, Sercom0Pad2, Sercom0Pad3};
use hal::target_device::{Peripherals};
use hal::target_device::gclk::clkctrl::GENR;
use hal::target_device::gclk::genctrl::SRCR;

use embedded_hal::blocking::serial::Write;

use rtfm::{app, Threshold};

macro_rules! dbgprint {
    ($($arg:tt)*) => {{}};
}

app! {
    device: hal,

    resources: {
        static BLUE_LED: hal::gpio::Pa17<hal::gpio::Output<hal::gpio::OpenDrain>>;
        static UART: Uart;
    },

    tasks: {
        SERCOM0: {
            path: uart,
            resources: [UART, BLUE_LED],
        },
    }
}

fn uart(t: &mut Threshold, mut r: SERCOM0::Resources) {
    r.BLUE_LED.toggle();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

// init::Peripherals contains two fields
// core: CorePeripherals
// device: Peripherals
fn init(mut p: init::Peripherals) -> init::LateResources {
    let mut pins = hal::pins(p.device.PORT);

    let mut clocks = GenericClockController::new(
        p.device.GCLK,
        &mut p.device.PM,
        &mut p.device.SYSCTRL,
        &mut p.device.NVMCTRL,
    );
    clocks.configure_gclk_divider_and_source(GENR::GCLK2, 1, SRCR::DFLL48M, false);
    let gclk2 = clocks.get_gclk(GENR::GCLK2).expect("Could not get clock 2");

    dbgprint!("Initializing serial port");

    let rx_pin: Sercom0Pad2 = pins.rx.into_pad(&mut pins.port);
    let tx_pin: Sercom0Pad3 = pins.tx.into_pad(&mut pins.port);
    let uart_clk = clocks.sercom0_core(&gclk2).expect("Could not configure sercom0 core clock");

    pins.rx_led.into_open_drain_output(&mut pins.port).set_low();
    let mut uart = Uart::new(&uart_clk, 9600.hz(), p.device.SERCOM0, &mut p.device.PM, tx_pin, rx_pin);

    let buffer: &[u8] = "Hello World".as_bytes();

    let mut tx_led = pins.tx_led.into_open_drain_output(&mut pins.port);
    let mut delay = Delay::new(p.core.SYST, &mut clocks);
    loop {
        uart.bwrite_all(buffer).expect("Failed to write to USART");
        tx_led.set_low();

        delay.delay_ms(200u8);
        tx_led.set_high();
    }

    dbgprint!("done init");
    init::LateResources {
        BLUE_LED: pins.led.into_open_drain_output(&mut pins.port),
        UART: uart,
    }
}
