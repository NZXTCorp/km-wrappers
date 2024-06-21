//! Memory-mapping of I/O address space.
//!
//! See [`MappedIoSpace`] for the main type handling mapping, unmapping, and giving access.

use crate::{private::Sealed, PhysicalAddress};
use bitflags::bitflags;
use core::{
    fmt::Debug,
    marker::PhantomData,
    mem::size_of,
    ptr::{read_volatile, write_volatile, NonNull},
};
use km_sys::{
    MmMapIoSpaceEx, MmUnmapIoSpace, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
    PAGE_NOCACHE, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOMBINE, SIZE_T, ULONG,
};

/// Helper struct to give volatile access to a [mapped I/O space](MappedIoSpace).
///
/// The lifetime parameter of this value binds it to the I/O space mapping it was derived from.
///
/// Note that volatile access does not guarantee any synchronization. I/O access is inherently
/// non-exclusive, so no synchronization is guaranteed, and data tearing may occur (see
/// [`MappedIoSpace::create_mapping`]'s safety documentation). Because no additional data integrity
/// is prescribed, both [read](VolatileAccess::read) and [write](VolatileAccess::write) access are
/// given through a shared reference, provided the selected access mode `A` allows that operation.
pub struct VolatileAccess<'a, T, A> {
    ptr: NonNull<T>,
    _access: PhantomData<A>,
    _tied_to: PhantomData<&'a ()>,
}

// manual implementation because the `A`ccess type is not necessarily `Debug` and we don't have
// perfect derive, yet
impl<T, A> Debug for VolatileAccess<'_, T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VolatileAccess")
            .field("ptr", &self.ptr)
            .finish()
    }
}

impl<'a, T, A> VolatileAccess<'a, T, A> {
    /// Returns the raw pointer to the mapped region.
    ///
    /// Note that this is *not* bound to the lifetime of this `VolatileAccess` value, so extreme
    /// caution has to be taken when using this pointer.
    pub fn ptr(&self) -> NonNull<T> {
        self.ptr
    }

    /// Creates a new `VolatileAccess` value for `U`, the type of a sub-value (e.g. field) of `T`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the pointer returned by `f` is derived from the original
    /// pointer passed in as `f`s argument (e.g. using [`core::ptr::addr_of_mut`]). The caller must
    /// also guarantee that the pointer returned follows the same rules regarding the pointed type
    /// `U` as the original pointer did regarding `T`. See the safety documentation of
    /// [`MappedIoSpace::create_mapping`] for those rules.
    pub unsafe fn map<U>(
        &self,
        f: impl FnOnce(NonNull<T>) -> NonNull<U>,
        // explicitly setting `'a` in the return type to only bind the lifetime to the original
        // lifetime of the `MappedIoSpace` value
    ) -> VolatileAccess<'a, U, A> {
        VolatileAccess {
            ptr: f(self.ptr),
            _tied_to: self._tied_to,
            _access: self._access,
        }
    }
}

impl<T: Copy, A: ReadAccess> VolatileAccess<'_, T, A> {
    /// Performs a volatile read.
    pub fn read(&self) -> T {
        // SAFETY: `VolatileAccess` inherits all necessary guarantees from `MappedIoSpace`
        // (`MappedIoSpace::create_mapping` in particular)
        unsafe { read_volatile(self.ptr.as_ptr()) }
    }
}

impl<T: Copy, A: WriteAccess> VolatileAccess<'_, T, A> {
    /// Performs a volatile write of the specified value.
    pub fn write(&self, value: T) {
        // SAFETY: `VolatileAccess` inherits all necessary guarantees from `MappedIoSpace`
        // (`MappedIoSpace::create_mapping` in particular)
        unsafe { write_volatile(self.ptr.as_ptr(), value) };
    }
}

impl<T: Copy, A: ReadAccess + WriteAccess> VolatileAccess<'_, T, A> {
    /// Performs a volatile read, applies `f` to the read value, then performs a volatile write of
    /// the applied value.
    pub fn modify(&self, f: impl FnOnce(T) -> T) {
        let value = f(self.read());
        self.write(value);
    }
}

/// Represents an I/O space region that is [mapped](MappedIoSpace::create_mapping) into memory
/// space.
///
/// Unmaps the region when dropped.
///
/// As no exclusive access to the I/O space can be guaranteed, only volatile access makes sense for
/// memory-mapped I/O. The [access](MappedIoSpace::access) method provides a wrapper around the raw
/// pointer that makes sure that all accesses are volatile, and that access is prevented once
/// this value is dropped.
#[repr(transparent)]
pub struct MappedIoSpace<T, A> {
    ptr: NonNull<T>,
    _access: PhantomData<A>,
}

// manual implementation because the `A`ccess type is not necessarily `Debug` and we don't have
// perfect derive, yet
impl<T, A> Debug for MappedIoSpace<T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MappedIoSpace")
            .field("ptr", &self.ptr)
            .finish()
    }
}

impl<T, A> MappedIoSpace<T, A> {
    /// Returns the raw pointer to the mapped region.
    ///
    /// Note that the returned pointer is *not* bound to the lifetime of this value, so extreme
    /// caution has to be taken when using this pointer.
    pub fn ptr(&self) -> NonNull<T> {
        self.ptr
    }
}

impl<T: Copy, A: Access> MappedIoSpace<T, A> {
    /// Maps space for the given `T` at the specified physical address to non-paged system space
    /// using the specified page protection.
    ///
    /// Returns `None` whenever no proper mapping could be established, in one of the following
    /// cases:
    ///
    /// - the space for mapping is insufficient (see MSDN docs in Remarks below)
    /// - the pointer returned wouldn't be aligned enough for `T`
    /// - `T` is zero-sized
    ///
    /// # Remarks
    ///
    /// The valid access types here are [`ReadOnly`], [`ReadWrite`], [`Execute`], [`ExecuteRead`]
    /// and [`ExecuteReadWrite`].
    ///
    /// See [`MmMapIoSpaceEx` on
    /// MSDN](https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-mmmapiospaceex)
    /// for more information about the underlying kernel API call.
    ///
    /// # Safety
    ///
    /// In order to provide a mostly safe interface, the caller must guarantee that
    ///
    /// - `physical_address` is valid for mapping a `T`-sized I/O space region
    /// - the access type `A` is valid for the desired mapping
    /// - values of `T` are valid even in the case of data tearing, which can happen for reads that
    ///   are bigger than what can be transferred with one machine instruction. (Usually one machine
    ///   word; `usize` sized). Effectively, `T` has to be valid for all byte combinations across
    ///   its size. For types smaller or equal to that machine word size this is not an issue. Here,
    ///   the caller only has to ensure that all reads from that region result in a valid `T` value.
    pub unsafe fn create_mapping(
        physical_address: PhysicalAddress,
        protection_modifiers: PageProtectionModifiers,
    ) -> Option<Self> {
        let size = size_of::<T>();

        if size == 0 {
            return None;
        }

        let page_protection = PageProtection {
            access: A::PROTECTION,
            modifiers: protection_modifiers,
        };

        // SAFETY: The caller provides all guarantees needed here.
        NonNull::new(unsafe {
            MmMapIoSpaceEx(physical_address, size as SIZE_T, page_protection.as_raw())
        })
        .and_then(|ptr| {
            // since `MmMapIoSpaceEx` always works on page boundaries, I don't think that this
            // pointer could ever be not aligned enough, but better safe than sorry
            if ptr.as_ptr().align_offset(core::mem::align_of::<T>()) == 0 {
                Some(MappedIoSpace {
                    ptr: ptr.cast(),
                    _access: PhantomData,
                })
            } else {
                // SAFETY: `ptr` comes straight from `MmMapIoSpaceEx`, and we're using the same size
                // as with that call.
                unsafe {
                    MmUnmapIoSpace(ptr.as_ptr(), size as SIZE_T);
                }
                None
            }
        })
    }

    /// Gives volatile access to the mapped region.
    pub fn access(&self) -> VolatileAccess<'_, T, A> {
        VolatileAccess {
            ptr: self.ptr,
            _tied_to: PhantomData,
            _access: PhantomData,
        }
    }
}

impl<T, A> Drop for MappedIoSpace<T, A> {
    fn drop(&mut self) {
        // SAFETY:
        // - We provide the same pointer and size that was initially returned by `MmMapIoSpaceEx`,
        //   fulfulling the API contract.
        // - The pointer is guaranteed to be valid, and `MmUnmapIoSpace` is guaranteed to only be
        //   called once by virtue of being a `Drop` implementation.
        unsafe {
            MmUnmapIoSpace(self.ptr.as_ptr().cast(), size_of::<T>() as SIZE_T);
        }
    }
}

/// Memory page protection settings for the `MmMapIoSpaceEx` function.
///
/// Only a subset of [all memory protection constants][memprot] are supported. See the
/// [documentation][fn-msdn] of `MmMapIoSpaceEx` for more information.
///
/// [fn-msdn]:
///     https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-mmmapiospaceex
/// [memprot]: https://docs.microsoft.com/en-us/windows/win32/memory/memory-protection-constants
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct PageProtection {
    access: PageProtectionOption,
    modifiers: PageProtectionModifiers,
}

impl PageProtection {
    fn as_raw(self) -> ULONG {
        (self.access as ULONG) | self.modifiers.bits()
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// Modifiers for how pages are mapped (see [`MappedIoSpace::create_mapping`]).
    pub struct PageProtectionModifiers: ULONG {
        /// Specifies non-cached memory.
        const PAGE_NOCACHE = PAGE_NOCACHE;
        /// Specifies write-combined memory (the memory should not be cached by the processor, but
        /// writes to the memory can be combined by the processor).
        const PAGE_WRITECOMBINE = PAGE_WRITECOMBINE;
    }
}

#[doc(hidden)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PageProtectionOption {
    /// The mapped range can only be read, not written.
    ReadOnly = PAGE_READONLY,
    /// The mapped range can be read or written.
    ReadWrite = PAGE_READWRITE,
    /// The mapped range can be executed, but not read or written.
    Execute = PAGE_EXECUTE,
    /// The mapped range can be executed or read, but not written.
    ExecuteRead = PAGE_EXECUTE_READ,
    /// The mapped range can be executed, read, or written.
    ExecuteReadWrite = PAGE_EXECUTE_READWRITE,
}

/// The mapped range can only be read, not written.
pub struct ReadOnly;
impl Sealed for ReadOnly {}

/// The mapped range can be read or written.
pub struct ReadWrite;
impl Sealed for ReadWrite {}

/// The mapped range can be executed, but not read or written.
pub struct Execute;
impl Sealed for Execute {}

/// The mapped range can be executed or read, but not written.
pub struct ExecuteRead;
impl Sealed for ExecuteRead {}

/// The mapped range can be executed, read, or written.
pub struct ExecuteReadWrite;
impl Sealed for ExecuteReadWrite {}

pub trait ReadAccess: Access {}
pub trait WriteAccess: Access {}

impl ReadAccess for ReadOnly {}
impl ReadAccess for ReadWrite {}
impl ReadAccess for ExecuteRead {}
impl ReadAccess for ExecuteReadWrite {}

impl WriteAccess for ReadWrite {}
impl WriteAccess for ExecuteReadWrite {}

pub trait Access: Sealed {
    const PROTECTION: PageProtectionOption;
}

impl Access for ReadOnly {
    const PROTECTION: PageProtectionOption = PageProtectionOption::ReadOnly;
}
impl Access for ReadWrite {
    const PROTECTION: PageProtectionOption = PageProtectionOption::ReadWrite;
}
impl Access for Execute {
    const PROTECTION: PageProtectionOption = PageProtectionOption::Execute;
}
impl Access for ExecuteRead {
    const PROTECTION: PageProtectionOption = PageProtectionOption::ExecuteRead;
}
impl Access for ExecuteReadWrite {
    const PROTECTION: PageProtectionOption = PageProtectionOption::ExecuteReadWrite;
}
