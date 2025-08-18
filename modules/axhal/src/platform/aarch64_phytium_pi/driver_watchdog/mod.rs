pub mod regs;
pub mod core;

pub use core::{SbsaGwdt, WDT0_BASE, WDT1_BASE, CLK_FREQ};
use kspin::SpinNoIrq;

pub static WDT0: SpinNoIrq<SbsaGwdt> = SpinNoIrq::new(SbsaGwdt {
    base: WDT0_BASE.as_usize(),
    clk: CLK_FREQ,
});

pub fn init_watchdog() {
    let mut wdt = WDT0.lock();
    wdt.init();
    debug!("看门狗模块初始化完成");
}