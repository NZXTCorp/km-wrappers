#![no_std]
#![deny(rust_2018_idioms)]
// `unsafe` blocks inside `unsafe` fns make sense
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::missing_safety_doc)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod assert;
pub mod io_mmap;
pub mod kdprint;
pub mod mode;
pub mod object_attributes;
pub mod panic;
pub mod port;
pub mod privileges;
pub mod wdf;

pub use km_shared as shared;
pub use km_sys::PHYSICAL_ADDRESS as PhysicalAddress;
pub use shared::utils::{AsRawMutPtr, AsRawPtr};

#[repr(transparent)]
pub struct DriverObjectHandle(km_sys::PDRIVER_OBJECT);
#[repr(transparent)]
pub struct UnicodeStringHandle(*mut shared::strings::UnicodeString);

/// This module/trait exists solely to augment other traits. When a trait extends from `Sealed`, it
/// cannot be implemented for types outside of this crate, as `Sealed` is not publicly accessible.
/// This allows external users to interact with and call trait methods, but prevents them from
/// creating their own implementations.
mod private {
    pub trait Sealed {}
}

pub(crate) use private::Sealed;
