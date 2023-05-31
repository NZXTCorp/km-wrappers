use super::{ffi::object_get_typed_context_worker, AsWdfReference};
use core::marker::PhantomData;
pub use km_sys::WDF_OBJECT_CONTEXT_TYPE_INFO;

/// Info for a user-defined context type associated to a WDF object.
///
/// On allocation of the context, its memory is guaranteed to be zero-initialized (see [MSDN]).
///
/// Context type info must be declared statically, which is done by using the
/// [`crate::declare_wdf_object_context_type!`] macro.
///
/// [MSDN]: https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/framework-object-context-space
#[repr(transparent)]
pub struct WdfObjectContextTypeInfo<T>(WDF_OBJECT_CONTEXT_TYPE_INFO, PhantomData<*mut T>);

// SAFETY: Needed to create statics of this type. There is no exposed safe way to create this type.
unsafe impl<T> Sync for WdfObjectContextTypeInfo<T> {}

impl<T> WdfObjectContextTypeInfo<T> {
    /// # Safety
    /// Not to be used directly. Use the [`crate::declare_wdf_object_context_type!`] macro instead.
    #[must_use]
    pub const unsafe fn _internal_new(info: WDF_OBJECT_CONTEXT_TYPE_INFO) -> Self {
        Self(info, PhantomData)
    }

    /// Retrieves a pointer to the object's context. On allocation of the context, its memory is
    /// guaranteed to be zero-initialized (see [MSDN][obj-context-space]).
    ///
    /// # Safety
    /// This function may only be called with an `object` whose context type in the object
    /// attributes was set to this type.
    ///
    /// There is no synchronization provided, nor is the lifetime of the pointee guaranteed. The
    /// caller has to ensure that the object this context is retrieved from is valid for the
    /// entirety of the context's usage.
    ///
    /// # Alignment
    ///
    /// From testing, the allocated context is always 16-bytes aligned on x64 (like `HeapAlloc` in
    /// user-mode, see its [Remarks][heap-alloc]), but I wasn't able to find any hard proof of this.
    ///
    /// [obj-context-space]: https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/framework-object-context-space
    /// [heap-alloc]: https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapalloc#remarks
    #[must_use]
    pub unsafe fn get(&self, object: &impl AsWdfReference) -> *mut T {
        // SAFETY: All the requirements to make this sound are moved onto the caller.
        unsafe { object_get_typed_context_worker(object.as_wdf_ref().upcast(), &self.0).cast() }
    }

    #[must_use]
    pub const fn as_ptr(&'static self) -> *const WDF_OBJECT_CONTEXT_TYPE_INFO {
        &self.0
    }
}

/// Declares a [`WdfObjectContextTypeInfo`] for the given type.
///
/// Example:
/// ```rs, ignore
/// declare_wdf_object_context_type! {
///     /// Docs
///     static BAR => MyContextType;
/// }
/// ```
#[macro_export]
macro_rules! declare_wdf_object_context_type {
    {
        $(#[$attr:meta])*
        $vis:vis static $accessor_name:ident => $t:ty;
    } => {
        $(#[$attr])*
        #[no_mangle]
        #[used]
        $vis static $accessor_name: $crate::wdf::context::WdfObjectContextTypeInfo<$t> =
            // SAFETY: Macro generated, correct initialization
            unsafe { $crate::wdf::context::WdfObjectContextTypeInfo::_internal_new(
                $crate::wdf::context::WDF_OBJECT_CONTEXT_TYPE_INFO {
                    Size: ::core::mem::size_of::<$crate::wdf::context::WDF_OBJECT_CONTEXT_TYPE_INFO>()
                        as u32,
                    ContextName: $crate::shared::cstrz!(::core::stringify!($t)).as_ptr() as *mut _,
                    ContextSize: ::core::mem::size_of::<$t>(),
                    UniqueType: &$accessor_name as *const _ as *const _,
                    EvtDriverGetUniqueContextType: None,
                }
            ) };
    };
}
