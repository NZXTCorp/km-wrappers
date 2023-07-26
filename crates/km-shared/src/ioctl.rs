use km_sys::{
    FILE_ANY_ACCESS, FILE_READ_DATA, FILE_WRITE_DATA, METHOD_BUFFERED, METHOD_IN_DIRECT,
    METHOD_NEITHER, METHOD_OUT_DIRECT,
};

/// Represents the method of transferring data to or from a device.
///
/// See [MSDN] for more information.
///
/// [MSDN]: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/defining-i-o-control-codes
#[repr(u8)]
pub enum IoCtlTransferType {
    Buffered = METHOD_BUFFERED as u8,
    /// Also known as `METHOD_DIRECT_TO_HARDWARE`
    InDirect = METHOD_IN_DIRECT as u8,
    /// Also known as `METHOD_DIRECT_FROM_HARDWARE`
    OutDirect = METHOD_OUT_DIRECT as u8,
    Neither = METHOD_NEITHER as u8,
}

impl IoCtlTransferType {
    const fn from_raw(value: u8) -> Self {
        match value as u32 {
            METHOD_BUFFERED => IoCtlTransferType::Buffered,
            METHOD_IN_DIRECT => IoCtlTransferType::InDirect,
            METHOD_OUT_DIRECT => IoCtlTransferType::OutDirect,
            METHOD_NEITHER => IoCtlTransferType::Neither,
            // function is not public, `from_raw` is only ever called on a two-bit value
            _ => panic!("raw ioctl transfertype out of bounds"),
        }
    }
}

bitflags::bitflags! {
    /// Represents the access rights the caller needs to be able to issue the I/O control code.
    pub struct IoCtlAccess: u8 {
        const READ_DATA = FILE_READ_DATA as u8;
        const WRITE_DATA = FILE_WRITE_DATA as u8;
    }
}

impl IoCtlAccess {
    pub const fn any_access() -> Self {
        const _: () = assert!(FILE_ANY_ACCESS == 0);

        Self::empty()
    }
}

/// An I/O Control code. See [MSDN] for more information.
///
/// [MSDN]: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/defining-i-o-control-codes
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IoControlCode(pub km_sys::ULONG);

impl IoControlCode {
    /// Creates a packed, non-Microsoft-defined I/O Control code. See [MSDN] for more information. This
    /// function mimicks the `CTL_CODE` macro from the WDK.
    ///
    /// This function panics if a device type number in the range 0-32767 (`0x0`-`0x7FFF`) is supplied.
    /// These are reserved for use by Microsoft.
    ///
    /// This function panics if a function code over 4095 (`0xFFF`) is supplied. This function also
    /// panics if a function code in the range 0-2047 (`0x0`-`0x7FF`) is supplied. These are reserved
    /// for use by Microsoft.
    ///
    /// Note that this is a `const fn`, so these panics happen at compile time if used to define IOCTL
    /// constants.
    ///
    /// [MSDN]: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/defining-i-o-control-codes
    pub const fn new_custom(
        device_type: u16,
        function: u16,
        method: IoCtlTransferType,
        access: IoCtlAccess,
    ) -> Self {
        assert!(device_type >= 0x8000, "`device_type` value is reserved");
        assert!(function >= 0x800, "`function` value is reserved");

        assert!(function <= 0xFFF, "`function` value is out of bounds");

        // Ported from the `CTL_CODE` macro in the WDK.
        let raw = ((device_type as u32) << 16)
            | ((access.bits() as u32) << 14)
            | ((function as u32) << 2)
            | (method as u32);
        Self(raw)
    }

    pub const fn device_type(self) -> u16 {
        (self.0 >> 16) as u16
    }

    pub const fn function(self) -> u16 {
        ((self.0 >> 2) & 0xFFF) as u16
    }

    pub const fn access(self) -> IoCtlAccess {
        IoCtlAccess::from_bits_truncate(((self.0 >> 14) & 0b11) as u8)
    }

    pub const fn method(self) -> IoCtlTransferType {
        IoCtlTransferType::from_raw((self.0 & 0b11) as u8)
    }
}

#[repr(transparent)]
pub struct TypedIoControlCode<I, O> {
    pub code: IoControlCode,
    _phantom: core::marker::PhantomData<(I, O)>,
}

impl<I, O> TypedIoControlCode<I, O> {
    pub const fn new(code: IoControlCode) -> Self {
        Self {
            code,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<I, O> PartialEq<IoControlCode> for TypedIoControlCode<I, O> {
    fn eq(&self, other: &IoControlCode) -> bool {
        self.code == *other
    }
}

impl<I, O> PartialEq<TypedIoControlCode<I, O>> for IoControlCode {
    fn eq(&self, other: &TypedIoControlCode<I, O>) -> bool {
        <Self as PartialEq<Self>>::eq(self, &other.code)
    }
}
