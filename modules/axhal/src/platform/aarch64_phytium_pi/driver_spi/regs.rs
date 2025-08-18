use safe_mmio::fields::{ReadOnly, ReadWrite, WriteOnly};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct PhytiumSpi {
    pub ctrl_r0: ReadWrite<u32>,     // SPICTRL0 (0x000): 控制寄存器0（数据位宽、时钟极性/相位）
    pub ctrl_r1: ReadWrite<u32>,     // SPICTRL1 (0x004): 控制寄存器1（连续接收数据量）
    pub ssienr: ReadWrite<u32>,      // SPIEN (0x008): 使能寄存器
    pub mwcr: ReadWrite<u32>,        // SPIMWCR (0x00C): Microwire 控制寄存器
    pub ser: ReadWrite<u32>,         // SPISER (0x010): 从机选择寄存器
    pub baudr: ReadWrite<u32>,       // SPIBAUDR (0x014): 波特率寄存器
    pub txftlr: ReadWrite<u32>,      // SPITXFTLR (0x018): 发送 FIFO 阈值
    pub rxftlr: ReadWrite<u32>,      // SPIRXFTLR (0x01C): 接收 FIFO 阈值
    pub txflr: ReadOnly<u32>,        // SPITXFLR (0x020): 发送 FIFO 等级
    pub rxflr: ReadOnly<u32>,        // SPIRXFLR (0x024): 接收 FIFO 等级
    pub sr: ReadOnly<u32>,           // SPISR (0x028): 状态寄存器
    pub imr: ReadWrite<u32>,         // SPIIMR (0x02C): 中断屏蔽寄存器
    pub isr: ReadOnly<u32>,          // SPIISR (0x030): 中断状态寄存器
    pub risr: ReadOnly<u32>,         // SPIRISR (0x034): 原始中断状态寄存器
    _resv0: [u8; 0x048 - 0x038],    // 保留至 0x048
    pub icr: WriteOnly<u32>,         // SPIICR (0x048): 中断清除寄存器
    pub dma_cr: ReadWrite<u32>,      // SPIDMACR (0x04C): DMA 控制寄存器
    _resv1: [u8; 0x060 - 0x050],    // 保留至 0x060
    pub dr: ReadWrite<u32>,          // SPIDR (0x060): 数据寄存器
    _resv2: [u8; 0x0FC - 0x064],    // 保留至 0x0FC
    pub rx_sample_dly: ReadWrite<u32>, // SPIRXSAMPLEDLY (0x0FC): 接收采样延迟
    pub cs: ReadWrite<u32>,          // SPICS (0x100): 片选寄存器
}