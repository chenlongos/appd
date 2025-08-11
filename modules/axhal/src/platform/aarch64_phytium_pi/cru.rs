use core::ptr::NonNull;
use tock_registers::{
    interfaces::Readable,
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};
use kspin::SpinNoIrq;
use crate::mem::{phys_to_virt, PhysAddr};
use tock_registers::interfaces::ReadWriteable;

// CRU 寄存器定义
register_structs! {
    pub CruRegs {
        (0x00 => _reserved0),
        (0x100 => cru_rst_ok: ReadWrite<u32, RST_OK::Register>),
        (0x104 => cru_rst_status: ReadOnly<u32, RST_STATUS::Register>),
        (0x108 => @END),
    }
}

register_bitfields![u32,
    RST_OK [
        SYSTEM_RESET OFFSET(0) NUMBITS(1) [], // 1=触发系统复位
        PERIPH_RESET OFFSET(1) NUMBITS(5) [], // bit 1-5: GPIO0~5 等外设
    ],
    RST_STATUS [
        DONE OFFSET(9) NUMBITS(1) [], // 1=复位完成
    ],
];

// CRU 控制器结构体
pub struct CruCtrl {
    regs: NonNull<CruRegs>,
}

unsafe impl Send for CruCtrl {}

impl CruCtrl {
    pub const fn new(base: *mut u8) -> Self {
        Self {
            regs: NonNull::new(base).unwrap().cast(),
        }
    }
    const fn regs(&self) -> &CruRegs {
        unsafe { self.regs.as_ref() }
    }
    const fn regs_mut(&mut self) -> &mut CruRegs {
        unsafe { self.regs.as_mut() }
    }
}

// API 实现
#[derive(Debug, Clone, Copy, Default)]
pub struct FResetConfig {
    pub instance_id: u32,
    pub base_address: usize,
}

#[derive(Clone, Copy)]
pub struct FResetCtrl {
    pub config: FResetConfig,
    pub is_ready: u32,
}

static CRU_CONFIG: [FResetConfig; 1] = [FResetConfig {
    instance_id: 0,
    base_address: 0x2800_0000usize,
}];

pub static CRU: SpinNoIrq<FResetCtrl> = SpinNoIrq::new(FResetCtrl {
    config: FResetConfig {
        instance_id: 0,
        base_address: 0,
    },
    is_ready: 0,
});

pub fn FResetInit(instance_p: &mut FResetCtrl, config_p: &FResetConfig) -> bool {
    assert!(Some(*instance_p).is_some() && Some(*config_p).is_some());
    let ret = true;
    if instance_p.is_ready == 0x11111111u32 {
        info!("CRU already initialized.");
        return false;
    }
    FResetDeInit(instance_p);
    instance_p.config = *config_p;
    instance_p.is_ready = 0x11111111u32;
    ret
}

pub fn FResetDeInit(instance_p: &mut FResetCtrl) -> bool {
    if instance_p.is_ready == 0 {
        return true;
    }
    instance_p.is_ready = 0;
    unsafe {
        core::ptr::write_bytes(instance_p as *mut FResetCtrl, 0, core::mem::size_of::<FResetCtrl>());
    }
    true
}

pub fn FResetLookupConfig(instance_id: u32) -> Option<FResetConfig> {
    if instance_id >= 1 {
        return None;
    }
    Some(CRU_CONFIG[instance_id as usize])
}

pub fn FResetSystem(instance_p: &mut FResetCtrl) -> bool {
    let base = instance_p.config.base_address;
    let cru = CruCtrl::new(phys_to_virt(PhysAddr::from(base)).as_mut_ptr());
    cru.regs().cru_rst_ok.modify(RST_OK::SYSTEM_RESET::SET);
    let mut timeout = 0;
    while cru.regs().cru_rst_status.read(RST_STATUS::DONE) != 1 && timeout < 500 {
        timeout += 1;
        crate::time::busy_wait(core::time::Duration::from_millis(1));
    }
    timeout < 500
}

pub fn FResetPeripheral(instance_p: &mut FResetCtrl, periph_id: u32) -> bool {
    if periph_id > 5 {
        return false;
    }
    let base = instance_p.config.base_address;
    let cru = CruCtrl::new(phys_to_virt(PhysAddr::from(base)).as_mut_ptr());
    cru.regs().cru_rst_ok.modify(RST_OK::PERIPH_RESET.val(periph_id));
    let mut timeout = 0;
    while cru.regs().cru_rst_status.read(RST_STATUS::DONE) != 1 && timeout < 500 {
        timeout += 1;
        crate::time::busy_wait(core::time::Duration::from_millis(1));
    }
    timeout < 500
}