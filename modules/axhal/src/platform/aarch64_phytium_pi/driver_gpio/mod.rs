pub mod regs;
pub mod core;

pub use core::{PhitiumGpio, GpioPins, BASE1};
use kspin::SpinNoIrq;

pub static GPIO: SpinNoIrq<PhitiumGpio> = SpinNoIrq::new(unsafe { PhitiumGpio::new(BASE1.as_usize()) });

pub fn init_gpio() {
    // 初始化 GPIO，设置默认配置
    let mut gpio = GPIO.lock();
    // 可选：设置默认方向或状态
    debug!("GPIO 初始化完成");
}