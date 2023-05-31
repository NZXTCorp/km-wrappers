//! Wrappers for accessing x86 I/O ports.

use core::sync::atomic::{compiler_fence, Ordering};

// Re-export these to expose non-fenced port access as well
pub use x86_64::{
    instructions::port::{
        PortGeneric, PortReadAccess, PortWriteAccess, ReadOnlyAccess, ReadWriteAccess,
        WriteOnlyAccess,
    },
    structures::port::{PortRead, PortWrite},
};

/// A wrapper for a port where all accesses are guaranteed to not be reordered by the compiler, via
/// [`compiler_fence`].
pub struct FencedPortGeneric<T, A>(PortGeneric<T, A>);

/// A wrapper for a read-write port where all accesses are guaranteed to not be reordered by the compiler, via
/// [`compiler_fence`].
pub type FencedPort<T> = FencedPortGeneric<T, ReadWriteAccess>;

/// A wrapper for a read-only port where all accesses are guaranteed to not be reordered by the compiler, via
/// [`compiler_fence`].
pub type FencedPortReadOnly<T> = FencedPortGeneric<T, ReadOnlyAccess>;

/// A wrapper for a write-only port where all accesses are guaranteed to not be reordered by the compiler, via
/// [`compiler_fence`].
pub type FencedPortWriteOnly<T> = FencedPortGeneric<T, WriteOnlyAccess>;

impl<T, A> FencedPortGeneric<T, A> {
    #[inline]
    pub const fn new(port: u16) -> Self {
        Self(PortGeneric::new(port))
    }
}

impl<T: PortRead, A: PortReadAccess> FencedPortGeneric<T, A> {
    /// Reads from the port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    #[inline]
    pub unsafe fn read(&mut self) -> T {
        compiler_fence(Ordering::SeqCst);
        // SAFETY: The caller is responsible for ensuring the soundness of this operation.
        let value = unsafe { self.0.read() };
        compiler_fence(Ordering::SeqCst);
        value
    }
}

impl<T: PortWrite, A: PortWriteAccess> FencedPortGeneric<T, A> {
    /// Writes to the port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    #[inline]
    pub unsafe fn write(&mut self, value: T) {
        compiler_fence(Ordering::SeqCst);
        // SAFETY: The caller is responsible for ensuring the soundness of this operation.
        unsafe { self.0.write(value) };
        compiler_fence(Ordering::SeqCst);
    }
}

impl<T, A> Clone for FencedPortGeneric<T, A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, A> PartialEq for FencedPortGeneric<T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T, A> Eq for FencedPortGeneric<T, A> {}
