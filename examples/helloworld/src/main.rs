#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

use core::time;

#[cfg(feature = "axstd")]
use axstd::println;

use axstd::thread::sleep;

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    println!("Hello, world!");
    let gpio0 = axhal::platform::gpio::PhitiumGpio::new(
        axhal::platform::gpio::phys_to_virt(axhal::platform::gpio::BASE1).into(),
    );
    let p = axhal::platform::gpio::GpioPins::p8;
    gpio0.set_pin_dir(p, true);
    let mut data = false;
    loop {
        sleep(time::Duration::from_secs(1));
        gpio0.set_pin_data(p, data);
        println!("current data: {data}");
        data = !data;
    }
}
