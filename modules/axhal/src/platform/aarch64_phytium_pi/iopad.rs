// iomux of phytium pi
use core::ptr::NonNull;

pub struct IoPadCtrl(NonNull<u32>);

pub struct IoReg0(NonNull<u32>);
pub struct IoReg1(NonNull<u32>);
impl IoPadCtrl {
    pub fn new(va: NonNull<u8>) -> Self {
        Self(va.cast())
    }

    pub fn reg0_by_offset(&self, offset: usize) -> Result<IoReg0, &'static str> {
        if offset > 0x24c || offset & 0x3 != 0 {
            Err("offset invalid")
        } else {
            Ok(IoReg0(unsafe { self.0.add(offset >> 2) }))
        }
    }

    pub fn reg1_by_offset(&self, offset: usize) -> Result<IoReg1, &'static str> {
        if offset < 0x1024 || offset > 0x124c || offset & 0x3 != 0 {
            Err("offset invalid")
        } else {
            Ok(IoReg1(unsafe { self.0.add(offset >> 2) }))
        }
    }
}

pub enum PollMode {
    None = 0b00,
    PollDown = 0b01 << 8,
    PollUp = 0b10 << 8,
}

impl IoReg0 {
    pub fn set_func(&mut self, func: u32) -> Result<(), &'static str> {
        if func >= 7 {
            Err("invalid func, must range 0-6")
        } else {
            unsafe {
                self.0
                    .write_volatile((self.0.read_volatile() & !0b111) | func);
            }
            Ok(())
        }
    }
    pub fn set_drive(&mut self, drive: u32) -> Result<(), &'static str> {
        if drive > 15 {
            Err("invalid drive mode, must range 0-15")
        } else {
            unsafe {
                self.0
                    .write_volatile((self.0.read_volatile() & !0b1111_0000) | (drive << 4));
            }
            Ok(())
        }
    }
    pub fn set_poll_mode(&mut self, pm: PollMode) -> Result<(), &'static str> {
        unsafe {
            self.0
                .write_volatile((self.0.read_volatile() & !0b11_0000_0000) | pm as u32);
        }
        Ok(())
    }
}

impl IoReg1 {
    fn fmt_value(&self, ps_336: u32, ps_100: u32) -> Result<u32, &'static str> {
        let ps_336 = ps_336 - 1;
        let ps_100 = ps_100 - 1;
        if ps_336 & !0b111 != 0 || ps_100 & !0b111 != 0 {
            return Err("param invalid");
        }

        return Ok(1 | (ps_100 << 1) | (ps_336 << 4));
    }
    /// 1 <= ps_336 <= 8, 1 <= ps_100 <= 8
    /// total delay = ps_336 * 336ps + ps_100 * 100ps
    pub fn set_input_delay(&mut self, ps_336: u32, ps_100: u32) -> Result<(), &'static str> {
        let v = self.fmt_value(ps_336, ps_100)?;
        unsafe {
            self.0.write_volatile((self.0.read_volatile() & !0xff) | v);
        }
        Ok(())
    }

    /// 1 <= ps_336 <= 8, 1 <= ps_100 <= 8
    /// total delay = ps_336 * 336ps + ps_100 * 100ps
    pub fn set_output_delay(&mut self, ps_336: u32, ps_100: u32) -> Result<(), &'static str> {
        let v = self.fmt_value(ps_336, ps_100)?;
        unsafe {
            self.0
                .write_volatile((self.0.read_volatile() & !0xff00) | v << 8);
        }
        Ok(())
    }
}
