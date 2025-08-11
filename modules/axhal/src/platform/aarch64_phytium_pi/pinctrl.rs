use core::ptr::NonNull;
use tock_registers::{
    interfaces::ReadWriteable,
    register_bitfields, register_structs,
    registers::ReadWrite,
};
use kspin::SpinNoIrq;
use crate::mem::{phys_to_virt, PhysAddr};

// PAD 寄存器定义（基址 0x32B30000）
register_structs! {
    pub PadRegs {
        (0x00 => _reserved0),
        (0x00D0 => x_reg0_scl: ReadWrite<u32, X_REG0::Register>),
        (0x00D4 => x_reg0_sda: ReadWrite<u32, X_REG0::Register>),
        (0x00D8 => x_reg0_gpio: ReadWrite<u32, X_REG0::Register>),
        (0x00DC => x_reg1_scl: ReadWrite<u32, X_REG1::Register>),
        (0x00E0 => x_reg1_sda: ReadWrite<u32, X_REG1::Register>),
        (0x00E4 => x_reg1_gpio: ReadWrite<u32, X_REG1::Register>),
        (0x00E8 => @END),
    }
}

register_bitfields![u32,
    X_REG0 [
        FUNC OFFSET(0) NUMBITS(4) [
            GPIO = 0,
            I2C = 5,
            UART = 1,
            CAN = 2,
            SPI = 3,
            PWM = 4,
        ],
        DRIVE_STRENGTH OFFSET(4) NUMBITS(3) [], // 2~12mA
        PULL OFFSET(7) NUMBITS(1) [], // 1=上拉
    ],
    X_REG1 [
        DELAY OFFSET(0) NUMBITS(4) [], // 100ps/366ps 粒度
    ],
];

// PAD 控制器结构体
pub struct PadCtrl {
    regs: NonNull<PadRegs>,
}

unsafe impl Send for PadCtrl {}

impl PadCtrl {
    pub const fn new(base: *mut u8) -> Self {
        Self {
            regs: NonNull::new(base).unwrap().cast(),
        }
    }
    const fn regs(&self) -> &PadRegs {
        unsafe { self.regs.as_ref() }
    }
    const fn regs_mut(&mut self) -> &mut PadRegs {
        unsafe { self.regs.as_mut() }
    }
}

// API 实现（复用 mio.rs 的 PhitiumMio）
use super::mio::PhitiumMio;

#[derive(Debug, Clone, Copy, Default)]
pub struct FIOPadConfig {
    pub instance_id: u32,
    pub base_address: usize,
}

#[derive(Clone,Copy)]
pub struct FIOPadCtrl {
    pub config: FIOPadConfig,
    pub is_ready: u32,
}

static PAD_CONFIG: [FIOPadConfig; 1] = [FIOPadConfig {
    instance_id: 0,
    base_address: 0x32B30000usize,
}];

pub static PAD: SpinNoIrq<FIOPadCtrl> = SpinNoIrq::new(FIOPadCtrl {
    config: FIOPadConfig {
        instance_id: 0,
        base_address: 0,
    },
    is_ready: 0,
});

pub fn FIOPadCfgInitialize(instance_p: &mut FIOPadCtrl, input_config_p: &FIOPadConfig) -> bool {
    assert!(Some(*instance_p).is_some() && Some(*input_config_p).is_some());
    let ret = true;
    if instance_p.is_ready == 0x11111111u32 {
        info!("PAD already initialized.");
        return false;
    }
    FIOPadDeInitialize(instance_p);
    instance_p.config = *input_config_p;
    instance_p.is_ready = 0x11111111u32;
    ret
}

pub fn FIOPadDeInitialize(instance_p: &mut FIOPadCtrl) -> bool {
    if instance_p.is_ready == 0 {
        return true;
    }
    instance_p.is_ready = 0;
    unsafe {
        core::ptr::write_bytes(instance_p as *mut FIOPadCtrl, 0, core::mem::size_of::<FIOPadCtrl>());
    }
    true
}

pub fn FIOPadLookupConfig(instance_id: u32) -> Option<FIOPadConfig> {
    if instance_id >= 1 {
        return None;
    }
    Some(PAD_CONFIG[instance_id as usize])
}

pub fn FIOPadSetFunc(instance_p: &mut FIOPadCtrl, offset: u32, func: u32) -> bool {
    if ![0x00D0, 0x00D4, 0x00D8].contains(&offset) {
        return false;
    }
    let base = instance_p.config.base_address;
    let mut pad = PadCtrl::new(phys_to_virt(PhysAddr::from(base)).as_mut_ptr());
    match offset {
        0x00D0 => pad.regs_mut().x_reg0_scl.modify(X_REG0::FUNC.val(func)),
        0x00D4 => pad.regs_mut().x_reg0_sda.modify(X_REG0::FUNC.val(func)),
        0x00D8 => pad.regs_mut().x_reg0_gpio.modify(X_REG0::FUNC.val(func)),
        _ => return false,
    }
    true
}

pub fn FMioFuncInit(instance_p: &mut PhitiumMio, func: u32) -> bool {
    match func {
        0 => instance_p.set_i2c(),
        1 => instance_p.set_uart(),
        _ => return false,
    }
    true
}

pub fn FMioFuncGetAddress(instance_p: &PhitiumMio, func: u32) -> u64 {
    let base = instance_p.get_func_raw() as u64;
    match func {
        0 | 1 => base, // I2C/UART 使用 MIO 基址
        _ => 0,
    }
}

pub fn FMioFuncGetIrqNum(_instance_p: &PhitiumMio, _func: u32) -> u32 {
    24 // UART/I2C 中断号
}