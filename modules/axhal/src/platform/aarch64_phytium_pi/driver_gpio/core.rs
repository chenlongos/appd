use super::regs::{PhitiumGpio, GpioPins};
use crate::mem::phys_to_virt;
use memory_addr::PhysAddr;

pub const BASE1: PhysAddr = PhysAddr::from(0x28035000);

impl PhitiumGpio {
    pub fn new(base: usize) -> &'static mut Self {
        let b = phys_to_virt(PhysAddr::from(base)).as_usize() as *mut PhitiumGpio;
        unsafe { &mut (*b) }
    }

    pub fn set_pin_dir(&mut self, pin: GpioPins, dir: bool) {
        let mut status = self.dir.0.bits();
        debug!("dir data = {status}");
        let pb = pin.bits();
        if dir {
            status |= pb;
        } else {
            status &= !pb;
        }
        debug!("dir data = {status}");
        self.dir.0 = GpioPins::from_bits_truncate(status);
    }

    pub fn set_pin_data(&mut self, pin: GpioPins, data: bool) {
        let mut status = self.data.0.bits();
        debug!("data = {status}");
        let pb = pin.bits();
        if data {
            status |= pb;
        } else {
            status &= !pb;
        }
        debug!("data = {status}");
        self.data.0 = GpioPins::from_bits_truncate(status);
    }
}