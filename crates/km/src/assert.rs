use km_sys::{KeGetCurrentIrql, APC_LEVEL, KIRQL};

/// Asserts that the IRQ level is low enough for the calling function to be paged.
///
/// See [When Should Code and Data Be Pageable?][MSDNPageable] for more information.
///
/// [MSDNPageable]: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/when-should-code-and-data-be-pageable-
#[inline(always)]
#[track_caller]
pub fn debug_assert_paged_code() {
    // SAFETY: FFI call; no further safety requirements
    debug_assert!(unsafe { KeGetCurrentIrql() } <= APC_LEVEL as KIRQL);
}
