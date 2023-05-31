#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::useless_transmute)]
#![allow(clippy::unnecessary_cast)]

mod generated;
pub use generated::*;

#[cfg(feature = "linking")]
const _: () = {
    // The linker includes below are the same, and in the same order as the C driver samples have them
    // in. The commented-out ones are not (yet) needed by `km`, but are left in for reference about the
    // order.

    // BufferOverflowFastFailK needs Windows 8+
    // see https://docs.microsoft.com/en-us/windows-hardware/drivers/develop/building-drivers-for-different-versions-of-windows
    // #[link(name = "BufferOverflowFastFailK")]
    // extern "C" {}

    // older lib for Vista+
    #[link(name = "BufferOverflowK")]
    extern "C" {}

    #[link(name = "ntoskrnl")]
    extern "C" {}

    // #[link(name = "hal")]
    // extern "C" {}
    // #[link(name = "wmilib")]
    // extern "C" {}

    #[link(name = "wdfldr")]
    extern "C" {}
    #[link(name = "wdfdriverentry")]
    extern "C" {}

    // #[link(name = "ntstrsafe")]
    // extern "C" {}

    #[link(name = "wdmsec")]
    extern "C" {}
};
