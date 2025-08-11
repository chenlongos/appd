use core::ptr::NonNull;
use tock_registers::{
    interfaces::{Readable, ReadWriteable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};
use kspin::SpinNoIrq;
use crate::mem::{phys_to_virt, PhysAddr};


register_structs! {
    pub ClockRegs {
        (0x00 => clk_con: ReadWrite<u32, CLK_CON::Register>),
        (0x04 => clk_div: ReadWrite<u32, CLK_DIV::Register>),
        (0x08 => clk_status: ReadOnly<u32, CLK_STATUS::Register>),
        (0x0c => @END),
    }
}

register_bitfields![u32,
    CLK_CON [
        ENABLE OFFSET(0) NUMBITS(1) [], // 1=使能时钟
        SOURCE OFFSET(1) NUMBITS(3) [], // 时钟源选择
    ],
    CLK_DIV [
        DIV OFFSET(0) NUMBITS(8) [], // 分频系数
    ],
    CLK_STATUS [
        READY OFFSET(0) NUMBITS(1) [], // 1=时钟准备好
    ],
];

// Clock 控制器结构体
pub struct ClockCtrl {
    regs: NonNull<ClockRegs>,
}

unsafe impl Send for ClockCtrl {}

impl ClockCtrl {
    pub const fn new(base: *mut u8) -> Self {
        Self {
            regs: NonNull::new(base).unwrap().cast(),
        }
    }
    const fn regs(&self) -> &ClockRegs {
        unsafe { self.regs.as_ref() }
    }
    const fn regs_mut(&mut self) -> &mut ClockRegs {
        unsafe { self.regs.as_mut() }
    }
}

// API 实现
#[derive(Debug, Clone, Copy, Default)]
pub struct FClockConfig {
    pub instance_id: u32,
    pub base_address: usize,
}

#[derive(Clone, Copy)]
pub struct FClockCtrl {
    pub config: FClockConfig,
    pub is_ready: u32,
}

static CLOCK_CONFIG: [FClockConfig; 1] = [FClockConfig {
    instance_id: 0,
    base_address: 0x2800_0000usize,
}];

pub static CLOCK: SpinNoIrq<FClockCtrl> = SpinNoIrq::new(FClockCtrl {
    config: FClockConfig {
        instance_id: 0,
        base_address: 0,
    },
    is_ready: 0,
});

pub fn FClockInit(instance_p: &mut FClockCtrl, config_p: &FClockConfig) -> bool {
    assert!(instance_p as *const _ as usize != 0 && config_p as *const _ as usize != 0);
    let ret = true;
    if instance_p.is_ready == 0x11111111u32 {
        info!("Clock already initialized.");
        return false;
    }
    FClockDeInit(instance_p);
    instance_p.config = *config_p;
    instance_p.is_ready = 0x11111111u32;
    ret
}

pub fn FClockDeInit(instance_p: &mut FClockCtrl) -> bool {
    if instance_p.is_ready == 0 {
        return true;
    }
    instance_p.is_ready = 0;
    unsafe {
        core::ptr::write_bytes(instance_p as *mut FClockCtrl, 0, core::mem::size_of::<FClockCtrl>());
    }
    true
}

pub fn FClockLookupConfig(instance_id: u32) -> Option<FClockConfig> {
    if instance_id >= 1 {
        return None;
    }
    Some(CLOCK_CONFIG[instance_id as usize])
}

pub fn FClockSetFreq(instance_p: &mut FClockCtrl, freq: u32) -> bool {
    let base = instance_p.config.base_address;
    let clock = ClockCtrl::new(phys_to_virt(PhysAddr::from(base)).as_mut_ptr());
    let sys_clk = 50000000; // 50MHz 系统时钟
    let div = sys_clk / freq;
    clock.regs().clk_div.modify(CLK_DIV::DIV.val(div));
    clock.regs().clk_con.modify(CLK_CON::ENABLE::SET);
    let mut timeout = 0;
    while clock.regs().clk_status.read(CLK_STATUS::READY) != 1 && timeout < 500 {
        timeout += 1;
        crate::time::busy_wait(core::time::Duration::from_millis(1));
    }
    timeout < 500
}

pub fn FClockGetFreq(instance_p: &mut FClockCtrl) -> u32 {
    let base = instance_p.config.base_address;
    let clock = ClockCtrl::new(phys_to_virt(PhysAddr::from(base)).as_mut_ptr());
    let sys_clk = 50000000; // 50MHz 系统时钟
    let div = clock.regs().clk_div.read(CLK_DIV::DIV);
    sys_clk / div
}