use core::ops::{Deref, DerefMut};

use super::*;
use crate::{
    DError,
    descriptor::{AdvRxDesc, AdvRxDescRead},
};
use alloc::sync::Arc;
use log::{error, trace};

struct RingInner {
    base: Ring<AdvRxDesc>,
}

impl RingInner {
    fn new(ring: Ring<AdvRxDesc>) -> Result<Self, DError> {
        Ok(Self { base: ring })
    }

    fn init(&mut self) -> Result<(), DError> {
        let bus_addr = self.bus_addr();
        let size_bytes = self.size_bytes();

        // Program the descriptor base address with the address of the region.
        self.reg_write(RDBAL, (bus_addr & 0xFFFFFFFF) as u32);
        self.reg_write(RDBAH, (bus_addr >> 32) as u32);

        // Set the length register to the size of the descriptor ring.
        self.reg_write(RDLEN, size_bytes as u32);

        let pkt_size_kb = self.pkt_size / 1024;

        // Program SRRCTL of the queue according to the size of the buffers and the required header handling.
        self.reg_write(
            SRRCTL,
            (SRRCTL::DESCTYPE::AdvancedOneBuffer + SRRCTL::BSIZEPACKET.val(pkt_size_kb as _)).value,
        );

        // If header split or header replication is required for this queue,
        // program the PSRTYPE register according to the required headers.
        // 暂时不需要头部分割

        self.reg_write(RDH, 0);
        self.reg_write(RDT, 0);

        // Enable the queue by setting RXDCTL.ENABLE. In the case of queue zero,
        // the enable bit is set by default - so the ring parameters should be set before RCTL.RXEN is set.
        // 使用推荐的阈值：PTHRESH=8, HTHRESH=8, WTHRESH=1
        self.enable_queue();

        // Poll the RXDCTL register until the ENABLE bit is set.
        // The tail should not be bumped before this bit was read as one.

        wait_for(
            || self.reg_read(RXDCTL) & RXDCTL::ENABLE::Enabled.value > 0,
            Duration::from_millis(1),
            Some(1000),
        )?;

        // Program the direction of packets to this queue according to the mode select in MRQC.
        // Packets directed to a disabled queue is dropped.
        // 暂时不配置 MRQC

        // Note: The tail register of the queue (RDT[n]) should not be bumped until the queue is enabled.
        // self.update_tail(self.descriptors.len() - 1);
        Ok(())
    }

    pub fn enable_queue(&mut self) {
        // 启用队列
        self.reg_write(
            RXDCTL,
            (RXDCTL::PTHRESH.val(8)
                + RXDCTL::HTHRESH.val(8)
                + RXDCTL::WTHRESH.val(1)
                + RXDCTL::ENABLE::Enabled)
                .value,
        );
    }

    pub fn disable_queue(&mut self) {
        // 禁用队列
        self.reg_write(
            RXDCTL,
            (RXDCTL::PTHRESH.val(8)
                + RXDCTL::HTHRESH.val(8)
                + RXDCTL::WTHRESH.val(1)
                + RXDCTL::ENABLE::Disabled)
                .value,
        );
    }

    // pub fn flush_descriptors(&mut self) {
    //     // 触发描述符写回刷新
    //     self.reg_write(
    //         RXDCTL,
    //         (RXDCTL::PTHRESH.val(8)
    //             + RXDCTL::HTHRESH.val(8)
    //             + RXDCTL::WTHRESH.val(1)
    //             + RXDCTL::ENABLE::Enabled
    //             + RXDCTL::SWFLUSH.val(1))
    //         .value,
    //     );
    // }

    /// 获取当前头部指针值
    pub fn get_head(&self) -> u32 {
        self.reg_read(RDH)
    }

    /// 获取当前尾部指针值
    pub fn get_tail(&self) -> u32 {
        self.reg_read(RDT)
    }

    /// 更新尾部指针
    pub fn update_tail(&mut self, mut tail: usize) {
        if tail == self.descriptors.len() {
            tail = 0;
        }
        self.reg_write(RDT, tail as u32);
    }
}
impl Deref for RingInner {
    type Target = super::Ring<AdvRxDesc>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for RingInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub struct RxRing(Arc<UnsafeCell<RingInner>>);

unsafe impl Send for RxRing {}

impl RxRing {
    #[allow(clippy::arc_with_non_send_sync)]
    pub(crate) fn new(idx: usize, mmio_base: NonNull<u8>, size: usize) -> Result<Self, DError> {
        let base = Ring::new(idx, mmio_base, size, PACKET_SIZE as usize)?;
        let mut ring_inner = RingInner::new(base)?;
        ring_inner.init()?;
        let ring = Arc::new(UnsafeCell::new(ring_inner));
        Ok(Self(ring))
    }

    fn this(&self) -> &RingInner {
        unsafe { &*self.0.get() }
    }
    fn this_mut(&mut self) -> &mut RingInner {
        unsafe { &mut *self.0.get() }
    }

    pub fn packet_size(&self) -> usize {
        self.this().pkt_size
    }

    pub fn next_pkt(&mut self) -> Option<RxPacket<'_>> {
        let index = self.next_index();
        let head = self.this().get_head() as usize;
        if head == index {
            return None; // 没有可用的缓冲区
        }
        let len;
        unsafe {
            let desc = &self.this().descriptors[index];
            // 检查描述符是否已完成
            if !desc.write.is_done() {
                trace!("RxRing: next_pkt descriptor not done at index: {index}");
                return None; // 描述符未完成，无法获取数据
            }
            len = desc.write.packet_length() as usize;
        }

        trace!("RxRing: next_pkt index: {index}");
        let request = self.this_mut().meta_ls[index]
            .request
            .take()
            .expect("Request should be set");

        Some(RxPacket {
            ring: self,
            request,
            len,
        })
    }

    pub fn submit(&mut self, request: Request) -> Result<(), DError> {
        let index = self.this_mut().get_tail() as usize;
        let ring = self.this_mut();
        if index + 1 == ring.get_head() as usize {
            error!("RxRing: submit no available buffer at index: {index}");
            return Err(DError::NoMemory); // 没有可用的缓冲区
        }

        // 更新描述符
        let desc = AdvRxDesc {
            read: AdvRxDescRead::new(request.bus_addr(), 0, false),
        };
        ring.descriptors.set(index, desc);
        ring.meta_ls[index].request = Some(request);

        // 更新尾部指针
        ring.update_tail(index + 1);

        Ok(())
    }

    fn next_index(&self) -> usize {
        let ring = self.this();
        (ring.get_tail() as usize + 1) % ring.count()
    }

    pub fn request_max_count(&self) -> usize {
        self.this().count() - 1
    }
}

impl Drop for RxRing {
    fn drop(&mut self) {
        // 在释放时禁用队列
        self.this_mut().disable_queue();
    }
}

pub struct RxPacket<'a> {
    pub request: Request,
    ring: &'a mut RxRing,
    len: usize,
}

impl<'a> RxPacket<'a> {
    pub fn re_submit(self) -> Result<(), DError> {
        self.ring.submit(self.request)
    }
}

impl Deref for RxPacket<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.request.deref()[..self.len]
    }
}
