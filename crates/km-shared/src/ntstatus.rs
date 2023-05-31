use core::{fmt::Display, num::NonZeroI32};
use km_sys::NTSTATUS;
use snafu::Snafu;

mod consts;

#[derive(Debug, Snafu, Clone, Copy, PartialEq, Eq)]
#[snafu(display("NTSTATUS {:X}", status))]
#[repr(transparent)]
pub struct NtStatusError {
    // Any non-success NTSTATUS cannot be 0.
    status: NonZeroI32,
}

impl NtStatusError {
    pub const fn status(&self) -> NtStatus {
        NtStatus(self.status.get())
    }

    pub(crate) const fn from_u32(status: u32) -> Self {
        match NtStatus::from_u32(status).result() {
            Ok(_) => panic!("not an error NTSTATUS"),
            Err(e) => e,
        }
    }
}

/// Represents an `NTSTATUS` success/error value.
///
/// See [Defining New NTSTATUS Values][MSDN] for more information.
///
/// [MSDN]:
///     https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/defining-new-ntstatus-values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct NtStatus(pub NTSTATUS);

impl NtStatus {
    pub const fn new(custom: bool, severity: Severity, facility: u16, code: u16) -> Self {
        assert!(
            facility <= 0x1FFF,
            "facility has only 13 bits - must be <= 0x1FFF"
        );

        let severity = (severity as u8 as u32) << 30;
        let custom = (custom as u32) << 29;
        let facility = (facility as u32) << 16;

        let status = severity | custom | facility | (code as u32);
        Self(status as i32)
    }

    pub const fn from_u32(status: u32) -> NtStatus {
        NtStatus(status as i32)
    }

    pub const fn severity(self) -> Severity {
        // see https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/using-ntstatus-values
        match (self.0 as u32) >> 30 {
            // 0x00000000..=0x3FFFFFFF
            0b00 => Severity::Success,
            // 0x40000000..=0x7FFFFFFF
            0b01 => Severity::Information,
            // 0x80000000..=0xBFFFFFFF
            0b10 => Severity::Warning,
            // 0xC0000000..=0xFFFFFFFF
            0b11 => Severity::Error,
            // We only have 2 bits to check, which we did above. This branch gets optimized out \o/
            _ => unreachable!(),
        }
    }

    pub const fn custom(self) -> bool {
        // see https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/defining-new-ntstatus-values
        (self.0 as u32) & (1 << 29) != 0
    }

    pub const fn facility(self) -> u16 {
        // see https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/defining-new-ntstatus-values
        // trim off top 3 bytes, then shift back and down to the bottom
        (((self.0 as u32) << 3) >> (3 + 16)) as u16
    }

    pub const fn code(self) -> u16 {
        // see https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/defining-new-ntstatus-values
        self.0 as u16
    }

    /// Converts an NtStatus to a Result, returning an error if the status is an error code. With
    /// debug assertions enabled, warnings are also treated as errors.
    pub const fn result(self) -> Result<NtStatus, NtStatusError> {
        let n = match self.severity() {
            Severity::Error => self.0,
            #[cfg(debug_assertions)]
            Severity::Warning => self.0,

            _ => return Ok(self),
        };

        if let Some(n) = NonZeroI32::new(n) {
            Err(NtStatusError { status: n })
        } else {
            // Any non-success NTSTATUS cannot be 0. The severity bits checked above are non-zero
            // for non success values, so this branch is unreachable and gets optimized out.
            unreachable!()
        }
    }
}

impl Display for NtStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:08X}", self.0)
    }
}

impl From<NtStatus> for NTSTATUS {
    fn from(status: NtStatus) -> NTSTATUS {
        status.0
    }
}

impl From<NTSTATUS> for NtStatus {
    fn from(status: NTSTATUS) -> NtStatus {
        NtStatus(status)
    }
}

impl From<u32> for NtStatus {
    fn from(status: u32) -> NtStatus {
        NtStatus::from_u32(status)
    }
}

/// Represents the severity of an `NTSTATUS` value.
///
/// See [`NtStatus::severity`].
// 2-bit field as part of an NTSTATUS, see https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/defining-new-ntstatus-values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Severity {
    Success = 0b00,
    Information = 0b01,
    Warning = 0b10,
    Error = 0b11,
}
