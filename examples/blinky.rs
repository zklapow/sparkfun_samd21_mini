#![feature(used)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate sparkfun_samd21_mini as hal;
extern crate panic_abort;

use hal::prelude::*;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::{CorePeripherals, Peripherals};
use rtfm::{app, Threshold, Resource};

macro_rules! dbgprint {
    ($($arg:tt)*) => {{}};
}

app! {
    device: hal,

    resources: {
        static BLUE_LED: hal::gpio::Pa17<hal::gpio::Output<hal::gpio::OpenDrain>>;
        static GREEN_LED: hal::gpio::Pa27<hal::gpio::Output<hal::gpio::OpenDrain>>;
        static YELLOW_LED: hal::gpio::Pb3<hal::gpio::Output<hal::gpio::OpenDrain>>;

        static BLINK_MODE: BlinkMode = BlinkMode::Off;
        static TIMER: hal::timer::TimerCounter3;
    },

    tasks: {
        TC3: {
            path: timer,
            resources: [TIMER, BLUE_LED, GREEN_LED, YELLOW_LED, BLINK_MODE],
        },
    }
}

pub enum BlinkMode {
    Off,
    Blue,
    Green,
    Yellow
}

fn timer(t: &mut Threshold, mut r: TC3::Resources) {
    if r.TIMER.wait().is_ok() {
        let mode = r.BLINK_MODE.borrow_mut(t);

        *mode = match *mode {
            BlinkMode::Off => {
                dbgprint!("LED Off");
                r.GREEN_LED.set_high(); // Active low
                r.BLUE_LED.set_low();
                r.YELLOW_LED.set_high(); // Active low
                BlinkMode::Blue
            }
            BlinkMode::Blue => {
                dbgprint!("Blue");
                r.BLUE_LED.set_high();
                BlinkMode::Green
            },
            BlinkMode::Green => {
                dbgprint!("Green");
                r.GREEN_LED.set_low();
                BlinkMode::Yellow
            },
            BlinkMode::Yellow => {
                dbgprint!("Yellow");
                r.YELLOW_LED.set_low();
                BlinkMode::Off
            }
        };
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn init(mut p: init::Peripherals, _r: init::Resources) -> init::LateResources {
    let mut clocks = GenericClockController::new(
        p.device.GCLK,
        &mut p.device.PM,
        &mut p.device.SYSCTRL,
        &mut p.device.NVMCTRL,
    );
    let gclk0 = clocks.gclk0();
    let mut pins = hal::pins(p.device.PORT);

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
        GREEN_LED: pins.tx_led.into_open_drain_output(&mut pins.port),
        YELLOW_LED: pins.rx_led.into_open_drain_output(&mut pins.port),
        BLUE_LED: pins.led.into_open_drain_output(&mut pins.port),
        TIMER: tc3,
    }
}
