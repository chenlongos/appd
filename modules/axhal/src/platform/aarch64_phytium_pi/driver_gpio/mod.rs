pub mod regs;
pub mod core;

pub use core::{BASE1, PhitiumGpioDrv};
pub use regs::GpioPins;
use kspin::SpinNoIrq;

pub static GPIO: SpinNoIrq<PhitiumGpioDrv> = SpinNoIrq::new(PhitiumGpioDrv::new(BASE1.as_usize()));

pub fn init_gpio() {
    // 初始化 GPIO，设置默认配置
    let _gpio = GPIO.lock();
    // 可选：设置默认方向或状态
    debug!("GPIO 初始化完成");
}