use core::panic::PanicInfo;
use km_sys::ULONG;

const BUGCHECK_RUST_PANIC: ULONG = u32::from_be_bytes(*b"Rust");

pub fn bugcheck_panic(info: &PanicInfo<'_>) -> ! {
    let (file, line, column) = info
        .location()
        .map(|l| (l.file().as_ptr(), l.line(), l.column()))
        .unwrap_or((core::ptr::null(), 0, 0));

    // SAFETY: FFI call. All parameters are just numbers, no additional requirements here.
    unsafe {
        km_sys::KeBugCheckEx(
            BUGCHECK_RUST_PANIC,
            file as u64,
            line as u64,
            column as u64,
            0,
        );
    }
}
