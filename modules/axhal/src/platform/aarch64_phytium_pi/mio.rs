use core::ptr::NonNull;

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};
register_structs! {
    PhitiumMioRegisters {
        (0x00 => _reserved0),
        (0x1000 => func_sel: ReadWrite<u32,FUNC_SEL::Register>),
        (0x1004=> func_sel_state: ReadOnly<u32,FUNC_SEL::Register>),
        (0x1008 => version:ReadOnly<u32>),
        (0x100c => @END),
    }
}

register_bitfields![u32,
    FUNC_SEL [
        SEL_STATE OFFSET(0) NUMBITS(2) [
            I2C = 0,
            UART = 1,
        ]
    ]
];

unsafe impl Send for PhitiumMio {}

pub struct PhitiumMio {
    base: NonNull<PhitiumMioRegisters>,
}

impl PhitiumMio {
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
        }
    }
    const fn regs(&self) -> &PhitiumMioRegisters {
        unsafe { self.base.as_ref() }
    }
    pub fn set_i2c(&mut self) {
        // debug!
        self.regs().func_sel.write(FUNC_SEL::SEL_STATE::I2C);
    }
    pub fn set_uart(&mut self) {
        // debug!
        self.regs().func_sel.write(FUNC_SEL::SEL_STATE::UART);
    }
    pub fn get_func_raw(&self) -> u32 {
        self.regs().func_sel_state.read(FUNC_SEL::SEL_STATE)
    }
    pub fn get_version(&self) -> u32 {
        self.regs().version.get()
    }
}

use crate::mem::PhysAddr;
use crate::mem::phys_to_virt;
const MIO_BASE0: PhysAddr = pa!(0x2801_4000);
const MIO_BASE1: PhysAddr = pa!(0x2801_6000);
const MIO_BASE2: PhysAddr = pa!(0x2801_8000);
const MIO_BASE3: PhysAddr = pa!(0x2801_a000);
const MIO_BASE4: PhysAddr = pa!(0x2801_c000);
const MIO_BASE5: PhysAddr = pa!(0x2801_e000);
const MIO_BASE6: PhysAddr = pa!(0x2802_0000);
const MIO_BASE7: PhysAddr = pa!(0x2802_2000);
const MIO_BASE8: PhysAddr = pa!(0x2802_4000);
const MIO_BASE9: PhysAddr = pa!(0x2802_6000);
const MIO_BASE10: PhysAddr = pa!(0x2802_8000);
const MIO_BASE11: PhysAddr = pa!(0x2802_a000);
const MIO_BASE12: PhysAddr = pa!(0x2802_c000);
const MIO_BASE13: PhysAddr = pa!(0x2802_e000);
const MIO_BASE14: PhysAddr = pa!(0x2803_0000);
const MIO_BASE15: PhysAddr = pa!(0x2803_2000);

use kspin::SpinNoIrq;

pub static MIO0: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE0).as_mut_ptr()));
pub static MIO1: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE1).as_mut_ptr()));
pub static MIO2: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE2).as_mut_ptr()));
pub static MIO3: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE3).as_mut_ptr()));
pub static MIO4: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE4).as_mut_ptr()));
pub static MIO5: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE5).as_mut_ptr()));
pub static MIO6: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE6).as_mut_ptr()));
pub static MIO7: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE7).as_mut_ptr()));
pub static MIO8: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE8).as_mut_ptr()));
pub static MIO9: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE9).as_mut_ptr()));
pub static MIO10: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE10).as_mut_ptr()));
pub static MIO11: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE11).as_mut_ptr()));
pub static MIO12: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE12).as_mut_ptr()));
pub static MIO13: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE13).as_mut_ptr()));
pub static MIO14: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE14).as_mut_ptr()));
pub static MIO15: SpinNoIrq<PhitiumMio> =
    SpinNoIrq::new(PhitiumMio::new(phys_to_virt(MIO_BASE15).as_mut_ptr()));
