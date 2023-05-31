use core::mem::size_of;
use km_sys::{UNICODE_STRING, WCHAR};

pub use wchar;

pub type UnicodeString = UNICODE_STRING;

pub const fn make_const_unicode_string<const N: usize>(s: &'static [WCHAR; N]) -> UnicodeString {
    let len_bytes = N * size_of::<WCHAR>();
    if len_bytes > u16::MAX as usize {
        panic!("`UNICODE_STRING`s only support a maximum length of `u16::MAX` bytes");
    }

    if !matches!(s.last(), Some(0)) {
        panic!("`UNICODE_STRING`s must be null terminated");
    }

    UnicodeString {
        Buffer: s.as_ptr() as *mut _,
        MaximumLength: len_bytes as u16,
        Length: (len_bytes - size_of::<WCHAR>()) as u16,
    }
}

#[macro_export]
macro_rules! cstrz {
    ($str:expr) => {
        concat!($str, "\0")
    };
}
