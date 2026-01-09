use aarch64_cpu::registers::CNTFRQ_EL0;
use aarch64_cpu::registers::Readable;

static mut GLOBAL_CLOCK: Option<Clock> = None;
#[derive(Debug, Clone, Copy)]
pub struct Clock {
	pub frequency: u32, // 系统时钟频率（Hz）
    pub div:u32,
    pub sys_frequency:u32,
}

impl Clock {
	/// 构造函数，自动从CNTFRQ_EL0寄存器读取频率
	pub fn new() -> Self {
		let freq = CNTFRQ_EL0.get() as u32;
		Clock { frequency: freq, div: 1, sys_frequency: freq }
	}

	/// 获取系统时钟频率（Hz）
	pub fn get_frequency(&self) -> u32 {
		self.frequency
	}

    pub fn set_frequency(&mut self, freq: u32) {
        self.frequency = freq;
        self.div = freq / self.sys_frequency;
    }

    pub fn get_div(&self) -> u32 {
        self.div
    }
    pub fn get_sys_frequency(&self) -> u32 {
        self.sys_frequency
    }

    pub fn init_global() {
        // 将基地址转换为NonNull<u8>
        let mut clock = Clock::new(); // 初始化时钟
        info!("Clock frequency: {} Hz", clock.get_frequency());
        clock.set_frequency(200_000_000);
        info!("Clock frequency: {} Hz", clock.get_frequency());

        // 存储到全局静态变量
        unsafe {
            GLOBAL_CLOCK = Some(clock);
        }
    }
}