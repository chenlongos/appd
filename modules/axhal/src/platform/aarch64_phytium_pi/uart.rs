use core::ptr::NonNull;

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

register_structs! {
    PhytiumUartRegs {
        /// Data Register.
        (0x00 => dr: ReadWrite<u32, DATA::Register>),
        (0x04 => _reserved0),
        /// Flag Register.
        (0x18 => fr: ReadOnly<u32, FLAG::Register>),
        (0x1c => _reserved1),
        ///
        (0x24 => tibd: ReadWrite<u32>),
        ///
        (0x28 => tfbd: ReadWrite<u32>),
        /// Control register.
        (0x2c => cr_h: ReadWrite<u32, CONTROLH::Register>),
        (0x30 => cr_l: ReadWrite<u32,CONTROLL::Register>),
        /// Interrupt FIFO Level Select Register.
        (0x34 => ifls: ReadWrite<u32>),
        /// Interrupt Mask Set Clear Register.
        (0x38 => imsc: ReadWrite<u32>),
        /// Raw Interrupt Status Register.
        (0x3c => ris: ReadOnly<u32>),
        /// Masked Interrupt Status Register.
        (0x40 => mis: ReadOnly<u32>),
        /// Interrupt Clear Register.
        (0x44 => icr: WriteOnly<u32>),
        (0x48 => @END),
    }
}

register_bitfields![u32,
    DATA [
        RAW OFFSET(0) NUMBITS(8),
        FE OFFSET(9) NUMBITS(1),
        PE OFFSET(10) NUMBITS(1),
        BE OFFSET(11) NUMBITS(1),
        OE OFFSET(12) NUMBITS(1),
    ],
    FLAG [
        CTS OFFSET(0) NUMBITS(1),
        DSR OFFSET(1) NUMBITS(1),
        DCD OFFSET(2) NUMBITS(1),
        BUSY OFFSET(3) NUMBITS(1),
        RXFE OFFSET(4) NUMBITS(1),
        TXFF OFFSET(5) NUMBITS(1),
        RXFF OFFSET(6) NUMBITS(1),
        TXFE OFFSET(7) NUMBITS(1),
    ],
    CONTROLH [
        BRK OFFSET(0) NUMBITS(1) [],
        PEN OFFSET(1) NUMBITS(1) [],
        EPS OFFSET(2) NUMBITS(1) [],
        STP2 OFFSET(3) NUMBITS(1) [],
        FEN OFFSET(4) NUMBITS(1) [],
        WLEN OFFSET(5) NUMBITS(2) [
            len5 = 0,
            len6 = 1,
            len7 = 2,
            len8= 3
        ],
        SPS OFFSET(7) NUMBITS(1) [],
    ],
    CONTROLL [
        ENABLE OFFSET(0) NUMBITS(1) [],
        RSV OFFSET(1) NUMBITS(7) [],
        TXE OFFSET(8) NUMBITS(1) [],
        RXE OFFSET(9) NUMBITS(1) [],
    ],
];

pub struct PhytiumUart {
    base: NonNull<PhytiumUartRegs>,
}

unsafe impl Send for PhytiumUart {}

impl PhytiumUart {
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
        }
    }
    fn get_ti_tf(clock_hz: u32, baude_rate: u32) -> (u32, u32) {
        let baude_rate_16 = 16 * baude_rate;
        let ti = clock_hz / baude_rate_16;
        let tf = clock_hz % baude_rate_16;
        let tf = (tf * 64 + (baude_rate_16 >> 1)) / baude_rate_16;
        (ti, tf)
    }
    /// no irq, no fifo, 8bits data, 1 stop bit, no odd-even check
    pub fn init_no_irq(&mut self, clock_hz: u32, baude_rate: u32) {
        // disable reg
        let regs = self.regs();
        regs.cr_l.write(CONTROLL::ENABLE::CLEAR);

        // set bd rate
        let (ti, tf) = Self::get_ti_tf(clock_hz, baude_rate);
        regs.tibd.set(ti);
        regs.tfbd.set(tf);

        // width 8 , no check, stop bit 1
        regs.cr_h.write(CONTROLH::WLEN::len8);

        // no interrupt
        regs.imsc.set(0);

        // enable uart ,rx, tx
        regs.cr_l
            .write(CONTROLL::ENABLE::SET + CONTROLL::TXE::SET + CONTROLL::RXE::SET);
    }
    const fn regs(&self) -> &PhytiumUartRegs {
        unsafe { self.base.as_ref() }
    }

    pub fn read_byte_poll(&self) -> u8 {
        while self.regs().fr.read(FLAG::RXFE) != 0 {}
        (self.regs().dr.get() & 0xff) as u8
    }

    pub fn put_byte_poll(&mut self, b: u8) {
        while self.regs().fr.read(FLAG::BUSY) == 1 || self.regs().fr.read(FLAG::TXFF) == 1 {}
        self.regs().dr.set(b as u32);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_mod() {
        let clock = 100_000_000;
        let bd_rate = 115200;
        let b16 = bd_rate * 16;
        let di = clock / b16;
        let df = clock % b16;
        let res = clock as f32 / b16 as f32;
        println!(
            "res = {res}, di = {di}, df={df},  df/b16 = {} , clock & (b16 -1)={}",
            df as f32 / b16 as f32,
            clock & (b16 - 1)
        );
        assert_eq!((54, 16), PhytiumUart::get_ti_tf(clock, bd_rate));
    }
}

use crate::mem::PhysAddr;
use crate::mem::phys_to_virt;
use kspin::SpinNoIrq;

/// we can only control uart 2
const UART2_BASE: PhysAddr = pa!(0x000_2800_E000);
pub static UART2: SpinNoIrq<PhytiumUart> =
    SpinNoIrq::new(PhytiumUart::new(phys_to_virt(UART2_BASE).as_mut_ptr()));
