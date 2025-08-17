use super::regs::{TIM_CTRL, PWM_CTRL};

const SYSTEM_CLK: u32 = 50_000_000; // 50MHz 系统时钟
const PWM_CONTROLLERS: usize = 8; // 8 个 PWM 控制器
const CHANNELS_PER_CONTROLLER: usize = 2; // 每个控制器 2 个通道
const GLOBAL_ENABLE_REG_ADDR: usize = 0x2807E020; // 全局使能寄存器

pub struct PwmConfig {
    pub frequency: u32, // PWM 频率 (Hz)
    pub duty_cycle: f32, // 占空比 (0.0-1.0)
    pub counting_mode: TIM_CTRL::MODE::Value,
    pub deadtime_ns: Option<u32>, // 死区时间 (ns)
    pub use_fifo: bool, // 是否使用 FIFO 模式
    pub output_behavior: PWM_CTRL::CMP::Value, // 输出行为
    pub initial_value: PWM_CTRL::ICOV::Value, // 初始输出值
}

pub struct PwmChannel {
    pub config: Option<PwmConfig>,
    pub enabled: bool,
}

pub struct PwmController {
    pub base: usize,
    pub channels: [PwmChannel; CHANNELS_PER_CONTROLLER],
}

pub struct PwmSystem {
    pub controllers: [PwmController; PWM_CONTROLLERS],
}