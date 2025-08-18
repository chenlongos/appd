use safe_mmio::fields::ReadWrite;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct SbsaGpio {
    pub wrr: ReadWrite<u32>,       // WDT_WRR (0x0000): 写操作重置计数器，读返回 0
    _resv1: [u8; 0x1000 - 0x004], // 保留至 0x1000
    pub wcs: ReadWrite<u32>,       // WDT_WCS (0x1000): 控制寄存器（使能信号等）
    pub wor: ReadWrite<u32>,       // WDT_WOR (0x1008): 清除寄存器
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SbsaGpioWcs {
    WDT_EN = 1 << 0, // Watchdog 使能信号，高有效
}