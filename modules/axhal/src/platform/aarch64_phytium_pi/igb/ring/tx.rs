use core::ops::{Deref, DerefMut};

use alloc::sync::Arc;
use log::trace;

use crate::descriptor::{TxAdvDescCmd, TxAdvDescType};

use super::*;
struct RingInner {
    base: Ring<AdvTxDesc>,
    finished: usize,
}

impl Deref for RingInner {
    type Target = super::Ring<AdvTxDesc>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for RingInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl RingInner {
    fn new(base: Ring<AdvTxDesc>) -> Self {
        Self { base, finished: 0 }
    }

    pub fn init(&mut self) -> Result<(), DError> {
        debug!("init tx");
        // Step 1: Allocate a region of memory for the transmit descriptor list
        // (Already done in Ring::new())
        let bus_addr = self.base.bus_addr();

        // Step 2: Program the descriptor base address with the address of the region
        self.reg_write(TDBAL, (bus_addr & 0xFFFFFFFF) as u32);
        self.reg_write(TDBAH, (bus_addr >> 32) as u32);

        // Step 3: Set the length register to the size of the descriptor ring
        let size_bytes = self.base.size_bytes();
        self.reg_write(TDLEN, size_bytes as u32);

        // Step 4: Program the TXDCTL register with the desired TX descriptor write back policy
        // Suggested values: WTHRESH = 1, all other fields 0
        self.reg_write(TXDCTL, TXDCTL::WTHRESH.val(1).value);

        self.reg_write(TDH, 0);
        self.reg_write(TDT, 0);

        // Step 5: If needed, set the TDWBAL/TWDBAH to enable head write back
        // (Not implemented in this basic version)

        // Step 6: Enable the queue using TXDCTL.ENABLE (queue zero is enabled by default)
        self.reg_write(
            TXDCTL,
            (TXDCTL::WTHRESH.val(1) + TXDCTL::ENABLE::Enabled).value,
        );

        // Step 7: Poll the TXDCTL register until the ENABLE bit is set
        wait_for(
            || self.reg_read(TXDCTL) & TXDCTL::ENABLE::Enabled.value > 0,
            Duration::from_millis(1),
            Some(1000),
        )?;

        // Note: The tail register of the queue (TDT[n]) should not be bumped until the queue is enabled
        // Step 8: Enable transmit path by setting TCTL.EN should be done only after all other settings are done
        // This is handled by the MAC layer through mac.enable_tx()
        debug!("TX ring initialized successfully");
        Ok(())
    }

    /// 获取当前头部指针值
    pub fn get_tx_head(&self) -> u32 {
        self.reg_read(TDH)
    }

    /// 获取当前尾部指针值
    pub fn get_tx_tail(&self) -> u32 {
        self.reg_read(TDT)
    }

    /// 发送单个数据包
    pub fn send_packet(&mut self, request: Request) -> Result<(), DError> {
        if request.buff.len() > PACKET_SIZE as usize {
            return Err(DError::InvalidParameter);
        }
        trace!("send {}", request.buff.len());
        request.buff.confirm_write_all();
        let tail = self.get_tx_tail() as usize;
        let next_tail = (tail + 1) % self.count();
        let head = self.get_tx_head() as usize;

        // 检查是否有空间
        if next_tail == head {
            return Err(DError::NoMemory); // 环形缓冲区已满
        }

        // 设置描述符
        let desc = AdvTxDesc::new(
            request.bus_addr(),
            request.buff.len(),
            TxAdvDescType::Data,
            &[
                TxAdvDescCmd::EOP,
                TxAdvDescCmd::RS,
                TxAdvDescCmd::IFCS,
                TxAdvDescCmd::DEXT,
            ],
        );

        self.descriptors.set(tail, desc);
        self.meta_ls[tail].request = Some(request);

        // 内存屏障确保描述符写入完成
        mb();

        // 更新尾部指针
        self.reg_write(TDT, next_tail as u32);

        Ok(())
    }

    fn next_finished(&mut self) -> Option<Request> {
        let head = self.get_tx_head() as usize;
        if self.finished == head {
            return None; // 没有新的完成描述符
        }
        let index = self.finished;

        trace!("next_finished index: {index}");

        // 检查描述符是否已完成
        unsafe {
            let desc = &self.descriptors[index];
            if !desc.write.is_done() {
                trace!("TxRing: next_finished descriptor not done at index: {index}");
                return None; // 描述符未完成，无法获取数据
            }
        }
        let request = self.meta_ls[index]
            .request
            .take()
            .expect("Request should be set");

        self.finished = (self.finished + 1) % self.count();
        Some(request)
    }
}

pub struct TxRing(Arc<UnsafeCell<RingInner>>);

unsafe impl Send for TxRing {}

impl TxRing {
    #[allow(clippy::arc_with_non_send_sync)]
    pub(crate) fn new(idx: usize, mmio_base: NonNull<u8>, size: usize) -> Result<Self, DError> {
        let mut ring_inner = RingInner::new(Ring::new(idx, mmio_base, size, PACKET_SIZE as usize)?);

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

    pub fn send(&mut self, request: Request) -> Result<(), DError> {
        self.this_mut().send_packet(request)
    }

    pub fn request_max_count(&self) -> usize {
        self.this().count() - 1
    }

    pub fn is_queue_full(&self) -> bool {
        let head = self.this().get_tx_head() as usize;
        let tail = self.this().get_tx_tail() as usize;
        (tail + 1) % self.this().count() == head
    }

    pub fn next_finished(&mut self) -> Option<Request> {
        self.this_mut().next_finished()
    }
}
