pub mod regs;
pub mod types;
pub mod core;

pub use core::{PwmSystem, PwmController, PwmConfig};
use kspin::SpinNoIrq;

pub static PWM_SYSTEM: SpinNoIrq<PwmSystem> = SpinNoIrq::new(PwmSystem::new());

pub fn init_pwm() {
    // Initialize PWM system and enable configured controllers
    let mut pwm_system = PWM_SYSTEM.lock();
    pwm_system.global_enable();
}