#![feature(used)]
#![feature(proc_macro)]
#![no_std]

extern crate atsamd21_hal as atsamd21;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate embedded_hal;
extern crate panic_abort;
extern crate sparkfun_samd21_mini as hal;

use embedded_hal::blocking::serial::Write;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::prelude::*;
use hal::sercom::{PadPin, Sercom0Pad2, Sercom0Pad3, Uart};
use hal::target_device::gclk::clkctrl::GENR;
use hal::target_device::gclk::genctrl::SRCR;
use hal::target_device::Peripherals;
use rtfm::{app, Threshold};

macro_rules! dbgprint {
    ($($arg:tt)*) => {{}};
}

app! {
    device: hal,

    resources: {
        static BLUE_LED: hal::gpio::Pa17<hal::gpio::Output<hal::gpio::OpenDrain>>;
        static TX_LED: hal::gpio::Pa27<hal::gpio::Output<hal::gpio::OpenDrain>>;
        static RX_LED: hal::gpio::Pb3<hal::gpio::Output<hal::gpio::OpenDrain>>;

        static UART: Uart;
        static TIMER: hal::timer::TimerCounter3;
    },

    tasks: {
        TC3: {
            path: int_tc3,
            resources: [TIMER, UART, RX_LED, TX_LED],
        },
        SERCOM0: {
            path: int_uart,
            resources: [UART, BLUE_LED]
        },
    }
}

const  buffer: &'static str = "Hello World";

fn int_tc3(t: &mut Threshold, mut r: TC3::Resources) {
    if r.TIMER.wait().is_ok() {
        for word in buffer.bytes() {
            match r.UART.write(word.clone()) {
                Ok(()) => {
                    r.TX_LED.set_low();
                }
                Err(_) => {
                    r.RX_LED.set_low();
                }
            }
        }

        r.TX_LED.set_high();
        r.RX_LED.set_high();
    }
}

fn int_uart(t: &mut Threshold, mut r: SERCOM0::Resources) {
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

    let mut led = pins.led.into_open_drain_output(&mut pins.port);
    led.set_low();

    let rx_pin: Sercom0Pad3 = pins.rx.into_pull_down_input(&mut pins.port).into_pad(&mut pins.port);
    let tx_pin: Sercom0Pad2 = pins.tx.into_push_pull_output(&mut pins.port).into_pad(&mut pins.port);
    let uart_clk = clocks.sercom0_core(&gclk2).expect("Could not configure sercom0 core clock");

    let mut uart = Uart::new(&uart_clk, 9600.hz(), p.device.SERCOM0, &mut p.core.NVIC,&mut p.device.PM, tx_pin, rx_pin);

    let mut rx_led = pins.rx_led.into_open_drain_output(&mut pins.port);
    let mut tx_led = pins.tx_led.into_open_drain_output(&mut pins.port);

    tx_led.set_low();
    rx_led.set_low();

    // Set up periodic sending
    let gclk0 = clocks.gclk0();

    let mut tc3 = hal::timer::TimerCounter::tc3_(
        &clocks.tcc2_tc3(&gclk0).unwrap(),
        p.device.TC3,
        &mut p.device.PM,
    );
    dbgprint!("start timer");
    tc3.start(1.hz());
    tc3.enable_interrupt();

    dbgprint!("done init");
    init::LateResources {
        BLUE_LED: led,
        TX_LED: tx_led,
        RX_LED: rx_led,
        UART: uart,
        TIMER: tc3,
    }
}
