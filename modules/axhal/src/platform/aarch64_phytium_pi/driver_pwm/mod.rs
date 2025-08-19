pub mod regs;
pub mod types;
pub mod core;

pub use core::{PwmSystem, PwmController};
pub use types::PwmConfig;
use kspin::SpinNoIrq;
use lazyinit::LazyInit;

pub static PWM_SYSTEM: LazyInit<SpinNoIrq<PwmSystem>> = LazyInit::new();

pub fn init_pwm() {
    // Initialize PWM system and enable configured controllers
    PWM_SYSTEM.init_once(SpinNoIrq::new(PwmSystem::new()));
    let pwm_system = PWM_SYSTEM.lock();
    pwm_system.global_enable();
}