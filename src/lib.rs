#![no_std]

extern crate atsamd21_hal as hal;

pub use hal::atsamd21g18a::*;
use hal::prelude::*;
pub use hal::*;

use gpio::{Floating, Input, Port};

/// Maps the pins to their arduino names and
/// the numbers printed on the board.
pub struct Pins {
    /// Opaque port reference
    pub port: Port,

    pub tx: gpio::Pa10<Input<Floating>>,
    pub rx: gpio::Pa11<Input<Floating>>,
    
    pub d2: gpio::Pa14<Input<Floating>>,
    pub d3: gpio::Pa9<Input<Floating>>,
    pub d4: gpio::Pa8<Input<Floating>>,

    /// Digital pin number 13, which is also attached to
    /// the red LED.  PWM capable.
    pub led: gpio::Pa17<Input<Floating>>,
    pub tx_led: gpio::Pa27<Input<Floating>>,
    pub rx_led: gpio::Pb3<Input<Floating>>,
}

pub fn pins(port: atsamd21g18a::PORT) -> Pins {
    let pins = port.split();

    Pins {
        port: pins.port,
        tx: pins.pa10,
        rx: pins.pa11,

        d2: pins.pa14,
        d3: pins.pa9,
        d4: pins.pa8,

        led: pins.pa17,
        tx_led: pins.pa27,
        rx_led: pins.pb3,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
