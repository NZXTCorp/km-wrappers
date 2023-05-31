//! Definitions and helpers for use in both kernel and user mode.

#![no_std]
#![deny(rust_2018_idioms)]
// `unsafe` blocks inside `unsafe` fns make sense
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::missing_safety_doc)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod ioctl;
pub mod ntstatus;
pub mod strings;
pub mod utils;

pub use wchar::wchz;
