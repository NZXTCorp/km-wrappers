use super::{NtStatus, NtStatusError};

impl NtStatus {
    pub const STATUS_SUCCESS: NtStatus = NtStatus::from_u32(0);
}

impl NtStatusError {
    pub const STATUS_ACCESS_DENIED: NtStatusError = NtStatusError::from_u32(0xC0000022);
    pub const STATUS_BUFFER_TOO_SMALL: NtStatusError = NtStatusError::from_u32(0xC0000023);
    pub const STATUS_INSUFFICIENT_RESOURCES: NtStatusError = NtStatusError::from_u32(0xC000009A);
    pub const STATUS_INTERNAL_ERROR: NtStatusError = NtStatusError::from_u32(0xC00000E5);
    pub const STATUS_INVALID_DEVICE_REQUEST: NtStatusError = NtStatusError::from_u32(0xC0000010);
    pub const STATUS_INVALID_PARAMETER: NtStatusError = NtStatusError::from_u32(0xC000000D);
    pub const STATUS_UNSUCCESSFUL: NtStatusError = NtStatusError::from_u32(0xC0000001);
}
