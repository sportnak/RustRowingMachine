/*!
  * run the main loop 10 times or so to get the baseline. Then use the baseline to estimate offsets
 */
#![no_std]
#![no_main]

use ufmt::{derive::uDebug, uwrite};
use arduino_hal::prelude::*;
use panic_halt as _;
use half::prelude::*;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let (mut spi, _) = arduino_hal::Spi::new(
        dp.SPI,
        pins.d13.into_output(),
        pins.d11.into_output(),
        pins.d12.into_pull_up_input(),
        pins.d10.into_output(),
        spi::Settings::default(),
    );
    
    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").void_unwrap();
    // Digital pin 0 is also connected to an onboard LED marked "L"
    let mut a0 = pins.a0.into_analog_input(&mut adc);

    let mut average_value = 0;
    let mut total_iterations: i16 = 0;
    let mut average_deviation = 0;
    let mut negative_deviation = 0;

    let mut max = 0;
    let mut min = 1023;
    loop {
        let voltage = a0.analog_read(&mut adc);
        let voltage = adc.read_blocking(&a0) as i16;
        if (voltage > max) {
            max = voltage;
        }

        if (voltage < min) {
            min = voltage;
        }

        nb::block!(spi.send(0b00001111)).void_unwrap();
        // Because MISO is connected to MOSI, the read data should be the same
        let data = nb::block!(spi.read()).void_unwrap();
    }
}
