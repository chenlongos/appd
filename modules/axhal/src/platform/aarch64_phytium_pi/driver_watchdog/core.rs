use super::regs::{SbsaGpio, SbsaGpioWcs};
use crate::mem::phys_to_virt;
use memory_addr::PhysAddr;

pub const WDT0_BASE: PhysAddr = PhysAddr::from(0x28040000);
pub const WDT1_BASE: PhysAddr = PhysAddr::from(0x28042000);
pub const CLK_FREQ: u64 = 100_000_000; // 100 MHz 时钟频率

pub struct SbsaGwdt {
    base: usize,
    clk: u64,
}

impl SbsaGwdt {
    pub fn new(base: usize) -> &'static mut SbsaGpio {
        let b = phys_to_virt(PhysAddr::from(base)).as_usize() as *mut SbsaGpio;
        unsafe { &mut (*b) }
    }

    pub fn init(&mut self) {
        let wdt = Self::new(self.base);
        wdt.wcs.set(0); // 禁用看门狗
        wdt.wor.set(0); // 清零清除寄存器
        debug!("看门狗初始化完成");
    }

    pub fn set_timeout(&mut self, timeout: u32) {
        let wdt = Self::new(self.base);
        let timeout_ms = timeout.min(wdt.wcs.get() / 1000); // 限制最大超时
        wdt.wor.set((self.clk * timeout_ms as u64 / 2) as u32); // 单阶段模式，WOR 设置为超时一半
        debug!("设置看门狗超时: {} 秒", timeout);
    }

    pub fn start(&mut self) {
        let wdt = Self::new(self.base);
        wdt.wcs.set(SbsaGpioWcs::WDT_EN as u32); // 使能看门狗
        debug!("看门狗已启动");
    }

    pub fn keepalive(&mut self) {
        let wdt = Self::new(self.base);
        wdt.wrr.set(0); // 写 WRR 重置计数器
        debug!("看门狗喂狗");
    }

    pub fn stop(&mut self) {
        let wdt = Self::new(self.base);
        wdt.wcs.set(0); // 禁用看门狗
        debug!("看门狗已停止");
    }
}