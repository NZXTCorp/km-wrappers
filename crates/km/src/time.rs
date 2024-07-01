use crate::mode::ProcessorMode;
use core::time::Duration;
use km_sys::{KeDelayExecutionThread, LARGE_INTEGER};

/// Sleep in kernel-mode, non-alertable.
///
/// > Where possible, Alertable should be set to FALSE and WaitMode should be set to KernelMode, in
/// > order to reduce driver complexity. The principal exception to this guideline is when the wait
/// > is a long-term wait.
pub fn sleep_km(d: Duration) {
    // the API needs units of 100ns.
    let ns100 = i64::try_from(
        d.as_secs()
            .saturating_mul(10_000_000)
            .saturating_add((d.subsec_nanos() / 10) as u64),
    )
    // Positive values mean that the sleep duration is converted to a date/time, meaning that it
    // will be affected by system time changes. Negative values mean that the sleep duration is
    // fully relative, and will not be affected by system time changes.
    .map(|v| v.saturating_neg())
    .unwrap_or(i64::MIN);

    let mut time = LARGE_INTEGER { QuadPart: ns100 };

    // SAFETY: Just an FFI call, nothing special here since both processor mode and alertability are pre-set.
    let _ = unsafe {
        KeDelayExecutionThread(ProcessorMode::KernelMode.into(), false.into(), &mut time)
    };
}
