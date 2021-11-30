/*!
 * Blink the builtin LED - the "Hello World" of embedded programming.
 */
 #![no_std]
 #![no_main]
 
 use ufmt::{derive::uDebug};
 use arduino_hal::prelude::*;
 use panic_halt as _;
 use half::prelude::*;

 const PRECISION: i32 = 1000;
 
 #[arduino_hal::entry]
 fn main() -> ! {
     let dp = arduino_hal::Peripherals::take().unwrap();
     let pins = arduino_hal::pins!(dp);
     let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
     let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
     
     ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").void_unwrap();
     // Digital pin 0 is also connected to an onboard LED marked "L"
     let mut a0 = pins.a0.into_analog_input(&mut adc);
 
     let mut average_value = MyFloat {
         main_value: 0,
         decimal_value: 0,
     };

     let mut total_iterations: i32 = 0;
     let mut average_deviation = MyFloat {
         main_value: 0,
         decimal_value: 0,
     };
     let mut negative_deviation = MyFloat {
         main_value: 0,
         decimal_value: 0,
     };

     let mut max = 0;
     let mut min = 1023;

     loop {
        let voltage = a0.analog_read(&mut adc);
        let voltage = adc.read_blocking(&a0) as i32;
        if (voltage > max) {
            max = voltage;
        }

        if (voltage < min) {
            min = voltage;
        }

        let voltage = MyFloat {
            main_value: voltage,
            decimal_value: 0
        };

        total_iterations = total_iterations + 1;
        
        ufmt::uwriteln!(&mut serial, "Test {:?} * {} / {} = {:?}",(sub_values(&voltage, &average_value).main_value % total_iterations), PRECISION, total_iterations, ((sub_values(&voltage, &average_value).main_value % total_iterations) * PRECISION / total_iterations));
        let avdev = divide_by_int(&sub_values(&voltage, &average_value), total_iterations);
        average_value = add_values(&average_value, &avdev);

        // if (average_value.main_value > 1000 && average_value.decimal_value == 0) {
            ufmt::uwriteln!(&mut serial, "Wow {}", total_iterations);
            ufmt::uwriteln!(&mut serial, "Voltage {:?}", voltage);
            ufmt::uwriteln!(&mut serial, "Avdev {:?}", avdev);
            ufmt::uwriteln!(&mut serial, "blech {:?}", add_values(&average_value, &avdev));
            ufmt::uwriteln!(&mut serial, "voltage {:?}, average_value {:?}\n", voltage, average_value);
        // }
        
        let deviation = sub_values(&voltage, &average_value);
        average_deviation = add_values(&average_deviation, &divide_by_int(&sub_values(&deviation, &average_deviation), total_iterations));

        if total_iterations > 10000 {
            ufmt::uwriteln!(&mut serial, "Average: {:?}", average_value);
            ufmt::uwriteln!(&mut serial, "Average Deviation: {:?}", average_deviation);
            ufmt::uwriteln!(&mut serial, "Max: {}, Min: {}", max, min);
            panic!("STOP");
        }
     }
 }

 fn divide_by_int(first: &MyFloat, second: i32) -> MyFloat {
    let leftover = ((first.main_value % second) * PRECISION / second);

     return resolve_float(MyFloat {
         main_value: first.main_value / second,
         decimal_value: leftover + ((first.decimal_value / second) / 10),
     });
 }

 fn add_values(first: &MyFloat, second: &MyFloat) -> MyFloat {
     return resolve_float(MyFloat {
        main_value: first.main_value + second.main_value,
        decimal_value: first.decimal_value + second.decimal_value,
     });
 }

 fn sub_values(first: &MyFloat, second: &MyFloat) -> MyFloat {
    return resolve_float(MyFloat {
        main_value: first.main_value - second.main_value,
        decimal_value: first.decimal_value - second.decimal_value,
    });
 }

 fn resolve_float(value: MyFloat) -> MyFloat {
    let decimal_value = value.decimal_value;
    let main_value = value.main_value;

    return MyFloat {
        main_value: if decimal_value > PRECISION { main_value + (decimal_value / PRECISION) } else if decimal_value < 0 { main_value + (decimal_value / PRECISION) - 1 } else { main_value },
        decimal_value: if decimal_value > PRECISION { decimal_value % PRECISION } else if decimal_value < 0 { PRECISION + decimal_value } else { decimal_value },
    }
 }

 #[derive(uDebug)]
 struct MyFloat {
     main_value: i32,
     decimal_value: i32,
 }

//  impl uDebug for MyFloat {
//      fn fmt(&self) -> str {
//          format!("{}.{}", self.main_value, self.decimal_value);
//      }
//  }
