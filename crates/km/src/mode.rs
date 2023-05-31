use core::hint::unreachable_unchecked;
use km_sys::{KPROCESSOR_MODE, MODE};

/// The processor mode, indicating where e.g. a request came from.
///
/// See e.g. [ExGetPreviousMode] for more information.
///
/// [ExGetPreviousMode]:
///     https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-exgetpreviousmode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum ProcessorMode {
    /// Kernel mode, skipping any priviliege checks.
    KernelMode = 0,
    /// User mode, validating any privilege checks.
    UserMode = 1,
}

impl ProcessorMode {
    pub(crate) unsafe fn from_kprocessor_mode_unchecked(mode: KPROCESSOR_MODE) -> Self {
        const _: () = assert!(MODE::MaximumMode.0 == 2);

        if mode == MODE::KernelMode.0 as _ {
            ProcessorMode::KernelMode
        } else if mode == MODE::UserMode.0 as _ {
            ProcessorMode::UserMode
        } else {
            // SAFETY: The const check above ensures that we are not missing any modes.
            unsafe { unreachable_unchecked() }
        }
    }
}

impl From<ProcessorMode> for KPROCESSOR_MODE {
    fn from(mode: ProcessorMode) -> Self {
        mode as i8
    }
}
