use super::regs::PhytiumSpi;
use crate::mem::phys_to_virt;
use memory_addr::PhysAddr;

pub const SPI0_BASE: PhysAddr = PhysAddr::from_usize(0x2803A000);
pub const CLK_FREQ: u64 = 100_000_000; // 100 MHz 时钟频率

pub struct PhytiumSpiDrv {
    pub base: usize,
    pub baudrate: u32,
    pub test_mode: bool,
}

impl PhytiumSpiDrv {
    pub fn new(base: usize) -> &'static mut PhytiumSpi {
        let b = phys_to_virt(PhysAddr::from(base)).as_usize() as *mut PhytiumSpi;
        unsafe { &mut (*b) }
    }

    pub fn init(&mut self, baudrate: u32, test_mode: bool) {
        let spi = Self::new(self.base);
        self.baudrate = baudrate;
        self.test_mode = test_mode;

        // 禁用 SPI
        spi.ssienr.0 = 0;
        // 配置波特率：baudr = Fclk / (2 * 所需频率)
        spi.baudr.0 = (CLK_FREQ / (2 * baudrate as u64)) as u32;
        // 配置控制寄存器0：主模式，模式0（CPOL=0, CPHA=0），8 位数据
        spi.ctrl_r0.0 = 0; // 清除配置
        if test_mode {
            spi.ctrl_r0.0 = spi.ctrl_r0.0 | (1 << 11); // 使能测试模式（MISO/MOSI 短接）
        }
        // 设置从机选择（默认 CS0）
        spi.ser.0 = 1;
        // 使能 SPI
        spi.ssienr.0 = 1;
        debug!("SPI 初始化完成，波特率: {}, 测试模式: {}", baudrate, test_mode);
    }

    pub fn send(&mut self, data: u8) {
        let spi = Self::new(self.base);
        while (spi.sr.0 & (1 << 2)) == 0 {} // 等待 TX FIFO 非满
        spi.dr.0 = data as u32;
        debug!("SPI 发送数据: 0x{:02x}", data);
    }

    pub fn recv(&mut self) -> u8 {
        let spi = Self::new(self.base);
        while (spi.sr.0 & (1 << 3)) == 0 {} // 等待 RX FIFO 非空
        let data = spi.dr.0 as u8;
        debug!("SPI 接收数据: 0x{:02x}", data);
        data
    }
}