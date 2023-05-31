use super::{context::WdfObjectContextTypeInfo, RawWdfObject, WdfObjectReference};
use super::{ExecutionLevel, SynchronizationScope};
use core::mem::{size_of, zeroed};
use km_sys::{ULONG, WDF_OBJECT_ATTRIBUTES};

#[repr(transparent)]
pub struct ObjectAttributes(pub(crate) WDF_OBJECT_ATTRIBUTES);

/// This is FFI-compatible with
/// [`km_sys::PFN_WDF_OBJECT_CONTEXT_CLEANUP`]/[`km_sys::PFN_WDF_OBJECT_CONTEXT_DESTROY`].
pub type ObjectEventCallback = unsafe extern "C" fn(object: WdfObjectReference<'_, RawWdfObject>);

impl ObjectAttributes {
    #[must_use]
    #[inline(always)] // analogous to how the `WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE` macro works
    pub fn new_with_context<T>(
        init: ObjectAttributesInit,
        context_type: &'static WdfObjectContextTypeInfo<T>,
    ) -> Self {
        let mut attributes = Self::new(init);
        attributes.0.ContextTypeInfo = context_type.as_ptr();
        attributes
    }

    #[must_use]
    #[inline(always)] // analogous to how the `WDF_OBJECT_ATTRIBUTES_INIT` macro works
    pub fn new(init: ObjectAttributesInit) -> Self {
        let ObjectAttributesInit {
            execution_level,
            synchronization_scope,
            object_cleanup_callback,
            object_destroy_callback,
        } = init;

        // SAFETY: The initialization mimicks the WDF macro `WDF_OBJECT_ATTRIBUTES_INIT`.
        let mut attributes = unsafe {
            let mut attributes: WDF_OBJECT_ATTRIBUTES = zeroed();
            attributes.Size = size_of::<WDF_OBJECT_ATTRIBUTES>() as ULONG;
            attributes
        };

        attributes.ExecutionLevel = execution_level;
        attributes.SynchronizationScope = synchronization_scope;
        attributes.EvtCleanupCallback =
            // SAFETY: `ObjectEventCallback` is defined to be compatible with the FFI function type.
            object_cleanup_callback.map(|f| unsafe { core::mem::transmute(f) });
        attributes.EvtDestroyCallback =
            // SAFETY: `ObjectEventCallback` is defined to be compatible with the FFI function type.
            object_destroy_callback.map(|f| unsafe { core::mem::transmute(f) });

        Self(attributes)
    }
}

impl Default for ObjectAttributes {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[must_use]
pub struct ObjectAttributesInit {
    pub execution_level: ExecutionLevel,
    pub synchronization_scope: SynchronizationScope,
    /// Object cleanup callback, see [MSDN].
    ///
    /// [MSDN]: https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfobject/nc-wdfobject-evt_wdf_object_context_cleanup
    pub object_cleanup_callback: Option<ObjectEventCallback>,
    /// Object destruction callback, see [MSDN].
    ///
    /// [MSDN]: https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfobject/nc-wdfobject-evt_wdf_object_context_cleanup
    pub object_destroy_callback: Option<ObjectEventCallback>,
    // this is missing fields, but they aren't needed at the moment
}

impl Default for ObjectAttributesInit {
    fn default() -> Self {
        Self {
            execution_level: ExecutionLevel::WdfExecutionLevelInheritFromParent,
            synchronization_scope: SynchronizationScope::WdfSynchronizationScopeInheritFromParent,
            object_cleanup_callback: None,
            object_destroy_callback: None,
        }
    }
}
