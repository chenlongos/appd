use core::{cell::UnsafeCell, ptr::NonNull, time::Duration};

use alloc::vec::Vec;
use dma_api::{DVec, Direction};
use futures::task::AtomicWaker;
use log::debug;
use mbarrier::mb;
use tock_registers::register_bitfields;

use crate::{
    Request,
    descriptor::{AdvTxDesc, Descriptor},
    err::DError,
    osal::wait_for,
};

mod rx;
mod tx;
pub use rx::{RxPacket, RxRing};
pub use tx::TxRing;

pub const DEFAULT_RING_SIZE: usize = 256;
const RDBAL: usize = 0xC000; // RX Descriptor Base Address Low
const RDBAH: usize = 0xC004; // RX Descriptor Base Address High
const RDLEN: usize = 0xC008; // RX Descriptor Length
const SRRCTL: usize = 0xC00C; // RX Descriptor Control
const RDH: usize = 0xC010; // RX Descriptor Head
const RDT: usize = 0xC018; // RX Descriptor Tail
const RXDCTL: usize = 0xC028; // RX Descriptor Control
// const RXCTL: usize = 0xC014; // RX Control
// const RQDPC: usize = 0xC030; // RX Descriptor Polling Control

// TX descriptor registers
const TDBAL: usize = 0xE000; // TX Descriptor Base Address Low
const TDBAH: usize = 0xE004; // TX Descriptor Base Address High
const TDLEN: usize = 0xE008; // TX Descriptor Length
const TDH: usize = 0xE010; // TX Descriptor Head
const TDT: usize = 0xE018; // TX Descriptor Tail
const TXDCTL: usize = 0xE028; // TX Descriptor Control
// const TDWBAL: usize = 0xE038; // TX Descriptor Write Back Address Low
// const TDWBAH: usize = 0xE03C; // TX Descriptor Write Back Address High

const PACKET_SIZE_KB: u32 = 2;
const PACKET_SIZE: u32 = PACKET_SIZE_KB * 1024;

register_bitfields! [
    // First parameter is the register width. Can be u8, u16, u32, or u64.
    u32,

    RDLEN [
        LEN OFFSET(7) NUMBITS(13)[],
    ],

    pub SRRCTL [
        BSIZEPACKET OFFSET(0) NUMBITS(7)[],
        BSIZEHEADER OFFSET(8) NUMBITS(4)[],
        RDMTS OFFSET(20) NUMBITS(5)[],
        DESCTYPE OFFSET(25) NUMBITS(3)[
            Legacy = 0b000,
            AdvancedOneBuffer = 0b001,
            AdvancedHeaderSplitting = 0b010,
            AdvancedHeaderReplicationAlways = 0b011,
            AdvancedHeaderReplicationLargePacket = 0b100,
        ],
        SECRC OFFSET(26) NUMBITS(1)[
            DoNotStrip = 0,
            Strip = 1,
        ],
        DROP_EN OFFSET(31) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
    ],

    pub RXDCTL [
        PTHRESH OFFSET(0) NUMBITS(5)[],
        HTHRESH OFFSET(8) NUMBITS(5)[],
        WTHRESH OFFSET(16) NUMBITS(5)[],
        ENABLE OFFSET(25) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
        SWFLUSH OFFSET(26) NUMBITS(1)[],
    ],

    pub TXDCTL [
        PTHRESH OFFSET(0) NUMBITS(5)[],
        HTHRESH OFFSET(8) NUMBITS(5)[],
        WTHRESH OFFSET(16) NUMBITS(5)[],
        ENABLE OFFSET(25) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
        SWFLUSH OFFSET(26) NUMBITS(1)[],
    ],


];

#[derive(Default)]
struct RingElemMeta {
    request: Option<Request>,
}

struct Ring<D: Descriptor> {
    pub descriptors: DVec<D>,
    ring_base: NonNull<u8>,
    _waker: AtomicWaker,
    meta_ls: Vec<RingElemMeta>,
    pkt_size: usize,
}

impl<D: Descriptor> Ring<D> {
    pub fn new(
        idx: usize,
        mmio_base: NonNull<u8>,
        size: usize,
        pkt_size: usize,
    ) -> Result<Self, DError> {
        let descriptors =
            DVec::zeros(size, 0x1000, Direction::Bidirectional).ok_or(DError::NoMemory)?;

        let ring_base = unsafe { mmio_base.add(idx * 0x40) };

        let mut meta_ls = Vec::with_capacity(size);
        for _ in 0..size {
            meta_ls.push(RingElemMeta::default());
        }

        Ok(Self {
            descriptors,
            ring_base,
            _waker: AtomicWaker::new(),
            meta_ls,
            pkt_size,
        })
    }

    pub fn bus_addr(&self) -> u64 {
        // 获取 DMA 物理地址
        // 暂时返回虚拟地址，这里需要根据实际的 DMA API 实现
        self.descriptors.bus_addr()
    }

    pub fn size_bytes(&self) -> usize {
        self.descriptors.len() * core::mem::size_of::<D>()
    }

    pub fn count(&self) -> usize {
        self.descriptors.len()
    }

    fn reg_addr(&self, reg: usize) -> NonNull<u32> {
        unsafe { self.ring_base.add(reg).cast() }
    }

    fn reg_write(&mut self, reg: usize, value: u32) {
        unsafe {
            self.reg_addr(reg).write_volatile(value);
        }
    }
    fn reg_read(&self, reg: usize) -> u32 {
        unsafe { self.reg_addr(reg).read_volatile() }
    }
}
