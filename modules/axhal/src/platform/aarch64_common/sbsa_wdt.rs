//! SBSA Watchdog Timer (WDT) driver for AArch64 platforms.

use kspin::SpinNoIrq;
use memory_addr::PhysAddr;

use crate::mem::phys_to_virt;
use aarch64_cpu::registers::CNTFRQ_EL0;
use aarch64_cpu::registers::Readable;
const WAT0_BASE: PhysAddr = pa!(axconfig::devices::WDT0_PADDR);

const SBSA_GWDT_WRR: usize = 0x0000;

const SBSA_GWDT_WCS: usize = 0x1000;
const SBSA_GWDT_WOR: usize = 0x1008;

struct SbsaWdt(usize);

static WDT: SpinNoIrq<SbsaWdt> =
    SpinNoIrq::new(SbsaWdt(phys_to_virt(WAT0_BASE).as_usize()));

/// Start Watchdog
pub fn start_watchdog() {
    let wdt = WDT.lock();
    unsafe {
        // Enable Watchdog Timer
        core::ptr::write_volatile(
            (wdt.0 + SBSA_GWDT_WCS) as *mut u32, 0x1);
    }
}

pub fn set_watchdog_timeout(timeout: u32) {
    let clk = CNTFRQ_EL0.get();
    let wdt = WDT.lock();
    unsafe {
        core::ptr::write_volatile(
            (wdt.0 + SBSA_GWDT_WOR) as *mut u32, (clk * timeout as u64) as u32);
    }
}

pub fn stop_watchdog() {
    let wdt = WDT.lock();
    unsafe {
        // Disable Watchdog Timer
        core::ptr::write_volatile(
            (wdt.0 + SBSA_GWDT_WCS) as *mut u32, 0x0);
    }
}

pub fn ping_watchdog() {
    let wdt = WDT.lock();
    unsafe {
        // Write to Watchdog Timer Reset Register to reset the timer
        core::ptr::write_volatile(
            (wdt.0 + SBSA_GWDT_WRR) as *mut u32, 0);
    }
}
