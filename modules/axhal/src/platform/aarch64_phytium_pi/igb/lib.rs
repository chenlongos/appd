#![no_std]

use core::{ops::Deref, ptr::NonNull};

use alloc::vec::Vec;
use dma_api::{DVec, Direction};
use log::debug;
pub use mac::{MacAddr6, MacStatus};
pub use trait_ffi::impl_extern_trait;

pub use crate::err::DError;
use crate::ring::DEFAULT_RING_SIZE;

extern crate alloc;

mod err;
mod mac;
#[macro_use]
pub mod osal;
mod descriptor;
mod phy;
mod ring;

pub use futures::{Stream, StreamExt};
pub use ring::{RxPacket, RxRing, TxRing};

pub struct Request {
    buff: DVec<u8>,
}

impl Request {
    fn new(buff: Vec<u8>, dir: Direction) -> Self {
        let buff = DVec::from_vec(buff, dir);
        Self { buff }
    }
    pub fn new_rx(buff: Vec<u8>) -> Self {
        Self::new(buff, Direction::FromDevice)
    }

    pub fn new_tx(buff: Vec<u8>) -> Self {
        Self::new(buff, Direction::ToDevice)
    }

    pub fn bus_addr(&self) -> u64 {
        self.buff.bus_addr()
    }
}

impl Deref for Request {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.buff.as_ref()
    }
}

pub struct Igb {
    mac: mac::Mac,
    phy: phy::Phy,
    _rx_ring_addrs: [usize; 16],
    _tx_ring_addrs: [usize; 16],
}

impl Igb {
    pub fn new(iobase: NonNull<u8>) -> Result<Self, DError> {
        let mac = mac::Mac::new(iobase);
        let phy = phy::Phy::new(mac);

        Ok(Self {
            mac,
            phy,
            _rx_ring_addrs: [0; 16],
            _tx_ring_addrs: [0; 16],
        })
    }

    pub fn open(&mut self) -> Result<(), DError> {
        self.mac.disable_interrupts();

        self.mac.reset()?;

        self.mac.disable_interrupts();

        debug!("reset done");

        let link_mode = self.mac.link_mode().unwrap();
        debug!("link mode: {link_mode:?}");
        self.phy.power_up()?;

        self.setup_phy_and_the_link()?;

        self.mac.set_link_up();

        self.phy.wait_for_auto_negotiation_complete()?;
        debug!("Auto-negotiation complete");
        self.config_fc_after_link_up()?;

        self.init_stat();

        self.mac.enable_interrupts();

        self.mac.enable_rx();
        self.mac.enable_tx();

        Ok(())
    }

    pub fn new_ring(&mut self) -> Result<(TxRing, RxRing), DError> {
        let tx_ring = TxRing::new(0, self.mac.iobase(), DEFAULT_RING_SIZE)?;
        let rx_ring = RxRing::new(0, self.mac.iobase(), DEFAULT_RING_SIZE)?;

        Ok((tx_ring, rx_ring))
    }

    fn config_fc_after_link_up(&mut self) -> Result<(), DError> {
        // TODO 参考 drivers/net/ethernet/intel/igb/e1000_mac.c
        // igb_config_fc_after_link_up
        Ok(())
    }

    fn setup_phy_and_the_link(&mut self) -> Result<(), DError> {
        self.phy.power_up()?;
        debug!("PHY powered up");
        self.phy.enable_auto_negotiation()?;

        Ok(())
    }

    pub fn read_mac(&self) -> MacAddr6 {
        self.mac.read_mac().into()
    }

    pub fn check_vid_did(vid: u16, did: u16) -> bool {
        // This is a placeholder for actual VID/DID checking logic.
        // In a real implementation, this would check the device's
        // vendor ID and device ID against the provided values.
        vid == 0x8086 && [0x10C9, 0x1533].contains(&did)
    }

    pub fn status(&self) -> MacStatus {
        self.mac.status()
    }

    pub fn enable_loopback(&mut self) {
        self.mac.enable_loopback();
    }

    pub fn disable_loopback(&mut self) {
        self.mac.disable_loopback();
    }

    fn init_stat(&mut self) {
        //TODO
    }

    /// # Safety
    /// This function should only be called from the interrupt handler.
    /// It will handle the interrupt by acknowledging
    pub unsafe fn handle_interrupt(&mut self) {
        let msg = self.mac.interrupts_ack();
        debug!("Interrupt message: {msg:?}");
        if msg.queue_idx & 0x1 != 0 {
            // let rx_ring = unsafe { &mut *(self.rx_ring_addrs[0] as *mut Ring<AdvRxDesc>) };
            // rx_ring.clean();
        }
    }

    pub fn irq_mode_legacy(&mut self) {
        self.mac.configure_legacy_mode();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Speed {
    Mb10,
    Mb100,
    Mb1000,
}
