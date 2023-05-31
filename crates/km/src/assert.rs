use core::arch::asm;
use km_sys::{APC_LEVEL, KIRQL};

/// Asserts that the IRQ level is low enough for the calling function to be paged.
///
/// See [When Should Code and Data Be Pageable?][MSDNPageable] for more information.
///
/// [MSDNPageable]: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/when-should-code-and-data-be-pageable-
#[inline(always)]
#[track_caller]
pub fn debug_assert_paged_code() {
    // SAFETY: FFI call; no further safety requirements
    debug_assert!(inlined_ke_get_current_irql() <= APC_LEVEL as KIRQL);
}

/// Retrieves the current IRQL; see [`KeGetCurrentIrql`][msdn] for more information.
///
/// This function is exported by `ntoskrnl.exe`, but apparently the import library `hal.lib` only
/// exports it in some versions (it does in 10.0.22000, it does not in 10.0.19041). `wdm.h` provides
/// this as a force-inlined function instead. This reimplements what that function does.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-kegetcurrentirql
#[inline(always)]
fn inlined_ke_get_current_irql() -> KIRQL {
    let mut irql: u64;

    // SAFETY: this matches the inlined header-only implementation of `KeGetCurrentIrql`
    // in the wdm.h, for x86_64.
    unsafe {
        asm!(
            "mov {0}, cr8",
            out(reg) irql,
            options(nomem, nostack)
        );
    }
    irql as KIRQL
}
