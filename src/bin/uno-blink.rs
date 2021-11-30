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
    
    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").void_unwrap();
    // Digital pin 0 is also connected to an onboard LED marked "L"
    let mut a0 = pins.a0.into_analog_input(&mut adc);

    let mut average_value = MyFloat {
        main_value: 0,
        first_precision: 0,
        second_precision: 0,
        third_precision: 0,
    };

    let mut total_iterations: i16 = 0;
    let mut average_deviation = MyFloat {
        main_value: 0,
        first_precision: 0,
        second_precision: 0,
        third_precision: 0,
    };
    let mut negative_deviation = MyFloat {
        main_value: 0,
        first_precision: 0,
        second_precision: 0,
        third_precision: 0,
    };

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

       let voltage = MyFloat {
           main_value: voltage,
           first_precision: 0,
           second_precision: 0,
           third_precision: 0,
       };

       total_iterations = total_iterations + 1;
       
       let avdev = divide_values_int(&sub_values(&voltage, &average_value), total_iterations);
       average_value = add_values(&average_value, &avdev);

       let deviation = sub_values(&voltage, &average_value);
       average_deviation = add_values(&average_deviation, &divide_values_int(&sub_values(&deviation, &average_deviation), total_iterations));
       if total_iterations > 10000 {
           ufmt::uwriteln!(&mut serial, "Average: {:?}", average_value);
           ufmt::uwriteln!(&mut serial, "Average Deviation: {:?}", average_deviation);
           ufmt::uwriteln!(&mut serial, "Max: {}, Min: {}", max, min);
           panic!("STOP");
       }
    }
}

fn divide_values_int(first: &MyFloat, second: i16) -> MyFloat {
    let remainder = first.main_value % second;
    let mut left = remainder;
    let mut right = second;
    let mut leftovers: [i16; 3] = [0; 3];
    for i in (0..3) {
        if (left < 10 && right < 10) {
            leftovers[i] = 0;
            continue;
        }
        if right > 3200 || left < (right * 10) {
            right = right / 10;
        } else {
            left = left / 10;
        }

        leftovers[i] = left / right % 10;
    }

    return resolve_float(MyFloat {
        main_value: first.main_value / second,
        first_precision: leftovers[0] + (first.first_precision / second),
        second_precision: leftovers[1] + (first.second_precision / second),
        third_precision: leftovers[2] + (first.third_precision / second),
    });
}

fn add_values(first: &MyFloat, second: &MyFloat) -> MyFloat {
    return resolve_float(MyFloat {
       main_value: first.main_value + second.main_value,
       first_precision: first.first_precision + second.first_precision,
       second_precision: first.second_precision + second.second_precision,
       third_precision: first.third_precision + second.third_precision,
    });
}

fn sub_values(first: &MyFloat, second: &MyFloat) -> MyFloat {
   return resolve_float(MyFloat {
       main_value: first.main_value - second.main_value,
       first_precision: first.first_precision - second.first_precision,
       second_precision: first.second_precision - second.second_precision,
       third_precision: first.third_precision - second.third_precision,
   });
}

fn resolve_float(value: MyFloat) -> MyFloat {
   let mut third_precision = value.third_precision;
   let mut second_precision = value.second_precision;
   let mut result = resolve_precision(second_precision, third_precision);
   second_precision = result.0;
   third_precision = result.1;

   let mut first_precision = value.first_precision;
   result = resolve_precision(first_precision, second_precision);
   first_precision = result.0;
   second_precision = result.1;

   let mut main_value = value.main_value;
   result = resolve_precision(main_value, first_precision);
   main_value = result.0;
   first_precision = result.1;

   return MyFloat {
       main_value,
       first_precision,
       second_precision,
       third_precision 
   }
}

fn resolve_precision(first: i16, second: i16) -> (i16, i16) {
   let mut first_prop = first;
   let mut second_prop = second;
   if (second <= -1 || second >= 1) {
       // second = 9 return first 
       first_prop = if second <= -1 && second % 10 != 0 { first + (second / 10) - 1 } else { first + (second / 10) };
       second_prop = if second <= -1 && second % 10 != 0 { (second % 10) + 10 } else { second % 10 };
   }

   return (first_prop, second_prop);
}

#[derive(uDebug)]
struct MyFloat {
    main_value: i16,
    first_precision: i16,
    second_precision: i16,
    third_precision: i16,
}

//  impl uDebug for MyFloat {
//      fn fmt(&self) -> str {
//          format!("{}.{}", self.main_value, self.first_precision);
//      }
//  }