use core::time::Duration;

use trait_ffi::def_extern_trait;

use crate::DError;

#[def_extern_trait]
pub trait Kernel {
    fn sleep(duration: Duration);
}

pub(crate) fn wait_for<F: FnMut() -> bool>(
    mut f: F,
    interval: Duration,
    try_count: Option<usize>,
) -> Result<(), DError> {
    for _ in 0..try_count.unwrap_or(usize::MAX) {
        if f() {
            return Ok(());
        }

        kernel::sleep(interval);
    }
    Err(DError::Timeout)
}
