use core::{ptr::NonNull, time::Duration};

use tock_registers::interfaces::ReadWriteable;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};

register_structs! {
    pub  TachoRegs {
        (0x0 => ctrl:ReadWrite<u32, CTRL::Register>),
        (0x4 => tach_result: ReadWrite<u32, TACH_RESULT::Register>),
        (0x8 => _reserve1),
        (0x1c => tacho_cycle: ReadWrite<u32>),
        (0x20 => _reserve2),
        (0x3c=>@END),
    }
}
register_bitfields![u32,
    CTRL [
        MODE OFFSET(0) NUMBITS(2) [
            TIMER = 0,
            TACHO = 1,
            CAPTURE = 2,
        ],
        SWR OFFSET(2) NUMBITS(1) [],
        COUNTER_MODE OFFSET(22) NUMBITS(1) [
            RESTART = 1,
            FREE_RUN  = 0,
        ],
        COUNTER_SERIES OFFSET(24) NUMBITS(1) [
            SER32 = 0,
            SER64 = 1,
        ],
        COUNTER_EN OFFSET(25) NUMBITS(1) [],
        COUNTER_CLR OFFSET(26) NUMBITS(1) [],
        TACH_MODE OFFSET(20) NUMBITS(2) [
            DOWN = 0,
            UP = 1,
            DOWN_UP = 2
        ],
        TACHO_EN  OFFSET(31) NUMBITS(1) []
    ],
    TACH_RESULT [
        VALUE OFFSET(0) NUMBITS(31) [],
        VALID OFFSET(31) NUMBITS(1) [],
    ]
];

// 5.25.2.7 tacho 功能寄存器配置序列
// 1. APB 写 ctrl_reg 寄存器，配置 mode 位为 01，选择 tachometer 功能；然后是计
// 数模式、计数器 32/64 位选择、tach 输入模式选择、tach 计数周期、时钟使能
// 配置、tach_in 消抖级数；
// 2. APB 写寄存器 ctrl_reg，使能计数器；
// 3. APB 写 intr_mask_n 寄存器使能对应的中断；
// 4. APB 写 tach_under_reg 和 tach_over_reg 值，设置合理转速范围；
// 5. APB 写寄存器 ctrl_reg，使能 tach_en；
// 6. APB 读 tach_result_reg 寄存器，bit[31]为 1 表示此时 bit[30:0]有效，表示
// 此时在设置的转速周期内的时钟计数。

pub struct Tacho {
    regs: NonNull<TachoRegs>,
}

impl Tacho {
    pub fn new(va: NonNull<u8>) -> Self {
        Self { regs: va.cast() }
    }
    pub fn regs(&self) -> &TachoRegs {
        unsafe { self.regs.as_ref() }
    }
    pub fn regs_mut(&mut self) -> &mut TachoRegs {
        unsafe { self.regs.as_mut() }
    }
    /// clock = 50M hz,
    pub fn init(&mut self) {
        self.regs_mut().ctrl.write(CTRL::SWR::SET);
        while self.regs().ctrl.read(CTRL::SWR) != 0 {}
        self.regs_mut().ctrl.modify(
            CTRL::MODE::TACHO
                + CTRL::COUNTER_SERIES::SER32
                + CTRL::TACH_MODE::DOWN_UP
                + CTRL::COUNTER_EN::SET
                + CTRL::TACHO_EN::SET,
        );
        // clock is 50M hz
        let d = 0x2faf07f; // copy from linux
        info!("set tacho cyle = {d}");
        self.regs_mut().tacho_cycle.set(d);
    }
    pub fn get_result(&self) -> Option<u32> {
        let res = self.regs().tach_result.get();
        info!("res = {res}");
        if res & TACH_RESULT::VALID::SET.value != 0 {
            Some(res & 0x7fff_ffff)
        } else {
            None
        }
    }
}
