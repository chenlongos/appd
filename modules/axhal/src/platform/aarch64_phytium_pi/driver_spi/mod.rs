pub mod regs;
pub mod core;

pub use core::{PhytiumSpiDrv, SPI0_BASE};
use kspin::SpinNoIrq;

pub static SPI0: SpinNoIrq<PhytiumSpiDrv> = SpinNoIrq::new(PhytiumSpiDrv {
    base: SPI0_BASE.as_usize(),
    baudrate: 0,
    test_mode: false,
});

pub fn init_spi() {
    let mut spi = SPI0.lock();
    spi.init(16_000_000, true); // 默认 16 MHz，启用测试模式
    debug!("SPI 模块初始化完成");
}