use crate::mode::ProcessorMode;
use km_sys::{LARGE_INTEGER, LUID};

pub struct Luid(LUID);

impl Luid {
    pub const SE_LOAD_DRIVER_PRIVILEGE: Self = Self::from_const(km_sys::SE_LOAD_DRIVER_PRIVILEGE);

    const fn from_const(raw: u32) -> Self {
        // The SE_* constants are actually i32/int, bindgen generates u32 though.
        let raw = raw as i32;

        let large_integer = LARGE_INTEGER {
            QuadPart: raw as i64,
        };

        // SAFETY: exact reimplementation of the header-only forced-inline `RtlConvertLongToLuid`.
        unsafe {
            Self(LUID {
                HighPart: large_integer.u.HighPart,
                LowPart: large_integer.u.LowPart,
            })
        }
    }
}

impl From<LUID> for Luid {
    fn from(luid: LUID) -> Self {
        Self(luid)
    }
}

pub fn check_single_privilege(privilege_luid: Luid, previous_mode: ProcessorMode) -> bool {
    // SAFETY: We call the function with the correct parameters.
    unsafe { km_sys::SeSinglePrivilegeCheck(privilege_luid.0, previous_mode.into()) != 0 }
}
