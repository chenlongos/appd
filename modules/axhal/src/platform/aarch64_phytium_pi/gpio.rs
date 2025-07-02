use bitflags::bitflags;
use safe_mmio::fields::ReadWrite;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C, align(4))]
pub struct PhitiumGpio {
    data: ReadWrite<GpioPins>,
    resv: ReadWrite<u16>,
    dir: ReadWrite<GpioPins>,
    resv2: ReadWrite<u16>,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
pub struct GpioPins(u16);

bitflags! {
    impl GpioPins: u16 {
        const p0 = 1<<0;
        const p1 = 1<<1;
        const p2 = 1<<2;
        const p3 = 1<<3;
        const p4 = 1<<4;
        const p5 = 1<<5;
        const p6 = 1<<6;
        const p7 = 1<<7;
        const p8 = 1<<8;
        const p9 = 1<<9;
        const p10 = 1<<10;
        const p11 = 1<<11;
        const p12 = 1<<12;
        const p13 = 1<<13;
        const p14 = 1<<14;
        const p15 = 1<<15;
    }
}

impl PhitiumGpio {
    pub fn new(base: usize) -> &'static mut Self {
        let b = base as *mut PhitiumGpio;
        unsafe { &mut (*b) }
    }
    pub fn set_pin_dir(&mut self, pin: GpioPins, dir: bool) {
        let mut status = self.dir.0.bits();
        debug!("dir data = {status}");
        let pb = pin.bits();
        if dir == true {
            status |= pb;
        } else {
            status &= !pb;
        }
        debug!("dir data = {status}");
        self.dir.0 = (GpioPins::from_bits_truncate(status));
    }
    pub fn set_pin_data(&mut self, pin: GpioPins, data: bool) {
        let mut status = self.dir.0.bits();
        debug!(" data = {status}");
        let pb = pin.bits();
        if data == true {
            status |= pb;
        } else {
            status &= !pb;
        }
        debug!(" data = {status}");
        self.data.0 = (GpioPins::from_bits_truncate(status));
    }
}
pub use crate::mem::phys_to_virt;
pub use memory_addr::PhysAddr;

pub const BASE1: PhysAddr = pa!(0x28035000);
