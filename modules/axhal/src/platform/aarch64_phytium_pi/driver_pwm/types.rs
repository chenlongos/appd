// Register types are no longer needed since we use primitive types

pub const SYSTEM_CLK: u32 = 50_000_000; // 50MHz 系统时钟
pub const PWM_CONTROLLERS: usize = 8; // 8 个 PWM 控制器
pub const CHANNELS_PER_CONTROLLER: usize = 2; // 每个控制器 2 个通道
pub const GLOBAL_ENABLE_REG_ADDR: usize = 0x2807E020; // 全局使能寄存器

#[derive(Clone, Copy)]
pub struct PwmConfig {
    pub frequency: u32, // PWM 频率 (Hz)
    pub duty_cycle: f32, // 占空比 (0.0-1.0)
    pub counting_mode: u32, // 计数模式 (0: Modulo, 1: UpAndDown)
    pub deadtime_ns: Option<u32>, // 死区时间 (ns)
    pub use_fifo: bool, // 是否使用 FIFO 模式
    pub output_behavior: u32, // 输出行为 (0-7: 不同的输出模式)
    pub initial_value: u32, // 初始输出值 (0 或 1)
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