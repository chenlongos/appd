#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

use core::time;

#[cfg(feature = "axstd")]
use axstd::println;

use axhal::misc::UART2;

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    println!("Hello, world!");
    let mut uart = UART2.lock();
    uart.init_no_irq(100_000_000, 115200);
    let mut data = 0;
    loop {
        uart.put_byte_poll(data);
        println!("send data {data}");
        let read = uart.read_byte_poll();
        println!("read data {read}");
        println!("sleep 1s");
        data = data.wrapping_add(1);
        axstd::thread::sleep(time::Duration::from_secs(1));
    }
}
