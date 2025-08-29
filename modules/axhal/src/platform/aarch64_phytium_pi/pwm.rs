// tacho timer of phytium pi

use crate::mem::{phys_to_virt, PhysAddr};
use core::ptr::NonNull;

// 添加全局静态PWM对象声明
static mut GLOBAL_PWM: Option<PwmCtrl> = None;

use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};
register_structs! {
    pub  PwmCtrlRegs {
        (0x0 => _reserve),
        (0x404 => tim_ctrl: ReadWrite<u32,CTRL::Register>),
        (0x408 => _reserve2),
        (0x40c => period: ReadWrite<u32, PERIOD::Register>),
        (0x410 => pwm_ctrl: ReadWrite<u32,PWM_CTRL::Register>),
        (0x414 => ccr: ReadWrite<u32,CCR::Register>),
        (0x418=>@END),
    }
}

register_bitfields![ u32,
    CTRL [
        SW_RST OFFSET(0) NUMBITS(1) [],
        ENABLE OFFSET(1) NUMBITS(1) [],
        DIV OFFSET(27) NUMBITS(12) [],
    ],
    PERIOD [
        CCR OFFSET(0) NUMBITS(16) []
    ],
    PWM_CTRL [
        MODE OFFSET(2) NUMBITS(1) [
            compare = 1,
        ],
        ICOV OFFSET(7) NUMBITS(1) [],
        CMP OFFSET(4) NUMBITS(3) [
            MATCH_1 = 0b000,
            MATCH_0 = 0b001,
            MATCH_REVERSE = 0b010,
            MATCH_1_00 = 0b011,
            MATCH_0_01 = 0b100,
        ]
    ],
    CCR [
        CCR OFFSET(0) NUMBITS(16) [],
        GPIO OFFSET(16) NUMBITS(1) [],
    ],
];

// 在上电复位完成后，进行寄存器配置，从而启动 Timer 和 PWM 功能：
// 1. APB 写 TIM_CTRL 寄存器，配置分频系数（PWM 控制器主时钟为 50MHz），计数
// 模式，全局中断使能开关，但是 TIM_CTRL.ENABLE 保持 0，即不使能 PWM。
// 2. APB 写 PWM_PERIOD 寄存器，设定计数周期。
// 3. APB 写 PWM_CTRL 寄存器，设定 PWM 工作模式(compare )，并根据工作模式，设
// 定相关模式工作参数。
// 4. 选择 compare 模式，则 APB 写 PWM_CCR 寄存器中写入 Duty 值(或向 FIFO 中)，
// 配置输出初始值（默认为 0）。
// 5. 配置中断使能位。
// APB 写 TIM_CTRL.ENABLE 值 1，该 PWM 可以工作。

pub struct PwmCtrl {
    regs: NonNull<PwmCtrlRegs>,
}

impl PwmCtrl {
    pub fn new(va: NonNull<u8>) -> Self {
        Self { regs: va.cast() }
    }
    fn regs(&self) -> &PwmCtrlRegs {
        unsafe { self.regs.as_ref() }
    }
    fn regs_mut(&mut self) -> &mut PwmCtrlRegs {
        unsafe { self.regs.as_mut() }
    }
    /// 50M clock, 2 分频
    /// t: cycle, default duty is 0.5
    pub fn init(&mut self) {
        let t = 10000;
        self.regs_mut().tim_ctrl.write(CTRL::SW_RST::SET);
        // wait rst done
        while self.regs().tim_ctrl.read(CTRL::SW_RST) != 0 {}
        self.regs_mut()
            .tim_ctrl
            .modify(CTRL::DIV.val(1) + CTRL::ENABLE::CLEAR);
        self.regs_mut().period.write(PERIOD::CCR.val(t as u32));
        self.regs_mut()
            .pwm_ctrl
            .modify(PWM_CTRL::MODE::compare + PWM_CTRL::CMP::MATCH_1_00);
        self.regs_mut().ccr.write(CCR::CCR.val((t >> 1) as u32));
        self.regs_mut().tim_ctrl.modify(CTRL::ENABLE::SET);
    }
    // duty from 1-100
    pub fn change_duty(&mut self, duty: u32) -> Result<(), &'static str> {
        match duty {
            1..=100 => {
                let t = self.regs().period.get();
                let nd = (100 - duty) * t / 100;
                self.regs_mut().ccr.write(CCR::CCR.val(nd));
                Ok(())
            }
            _ => Err("duty must range between 1-100"),
        }
    }
    /// 初始化全局PWM实例
    pub fn init_global() {
        // 将基地址转换为NonNull<u8>
        let base_addr = 0x2804a000;
        let pwm_va =
            unsafe { NonNull::new_unchecked(phys_to_virt(PhysAddr::from(base_addr)).as_mut_ptr()) };
        // 创建PWM实例并初始化
        let mut pwm = PwmCtrl::new(pwm_va);
        pwm.init();

        // 存储到全局静态变量
        unsafe {
            GLOBAL_PWM = Some(pwm);
        }
    }

    /// 获取全局PWM实例
    pub fn global() -> Option<&'static mut PwmCtrl> {
        unsafe { GLOBAL_PWM.as_mut() }
    }
}
