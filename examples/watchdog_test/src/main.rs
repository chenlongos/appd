#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    println!("Hello, world!");
    let mut count = 1usize;
    loop {
        println!("count {count}");
        count += 1;
        axstd::thread::sleep(core::time::Duration::from_secs(1));
    }
}
