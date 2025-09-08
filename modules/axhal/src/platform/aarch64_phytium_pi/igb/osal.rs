use core::time::Duration;
use crate::time::busy_wait;

use super::DError;

pub(crate) fn wait_for<F: FnMut() -> bool>(
    mut f: F,
    interval: Duration,
    try_count: Option<usize>,
) -> Result<(), DError> {
    for _ in 0..try_count.unwrap_or(usize::MAX) {
        if f() {
            return Ok(());
        }

        busy_wait(interval);
    }
    Err(DError::Timeout)
}
