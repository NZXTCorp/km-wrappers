use super::{
    ffi::{object_dereference_actual, object_reference_actual},
    RawWdfObject,
};
use crate::Sealed;
use core::{marker::PhantomData, ptr::null_mut};
use km_sys::WDFOBJECT;

#[derive(Debug)]
#[repr(transparent)]
pub struct WdfObjectReference<'a, T: 'static>(WDFOBJECT, PhantomData<&'a T>);
impl<T> Sealed for WdfObjectReference<'_, T> {}

impl<T> Clone for WdfObjectReference<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for WdfObjectReference<'_, T> {}

impl<T> WdfObjectReference<'_, T> {
    pub(crate) fn raw(&self) -> *mut T {
        self.0.cast()
    }

    pub(crate) fn raw_obj(&self) -> WDFOBJECT {
        self.0
    }

    pub fn to_owned(&self) -> OwnedWdfObject<T> {
        // SAFETY: We're calling the function with a guaranteed valid handle, and the rest is set to
        // sane/null defaults.
        unsafe { object_reference_actual(self.0.cast(), null_mut(), 0, null_mut()) }

        OwnedWdfObject {
            raw: WdfObjectReference(self.0, PhantomData),
        }
    }

    /// Upcast to a generic WDF object reference.
    pub fn upcast(&self) -> WdfObjectReference<'_, RawWdfObject> {
        WdfObjectReference(self.0, PhantomData)
    }
}

impl WdfObjectReference<'_, RawWdfObject> {
    /// Converts the generic WDF object reference to a reference to a specific type.
    ///
    /// # Safety
    /// The caller must ensure that the object is actually of the type `T`.
    pub unsafe fn downcast<T>(&self) -> WdfObjectReference<'_, T> {
        WdfObjectReference(self.0, PhantomData)
    }
}

impl<T> AsWdfReference for WdfObjectReference<'_, T> {
    type ObjectType = T;

    fn as_wdf_ref(&self) -> WdfObjectReference<'_, Self::ObjectType> {
        *self
    }
}

/// Represents an owned WDF object. See [Framework Object Life Cycle][msdn] for more details.
///
/// [msdn]: https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/framework-object-life-cycle
#[derive(Debug)]
#[repr(transparent)]
pub struct OwnedWdfObject<T: 'static> {
    raw: WdfObjectReference<'static, T>,
}
impl<T> Sealed for OwnedWdfObject<T> {}

impl<T> OwnedWdfObject<T> {
    /// Converts a raw handle from a `WdfXCreate` function to an `OwnedWdfObject`.
    pub(crate) fn from_new_raw(obj: *mut T) -> Self {
        WdfObjectReference(obj.cast(), PhantomData).to_owned()
    }

    pub fn as_ref(&self) -> WdfObjectReference<'_, T> {
        WdfObjectReference(self.raw.0, PhantomData)
    }
}

impl<T> Clone for OwnedWdfObject<T> {
    fn clone(&self) -> Self {
        (self.raw).to_owned()
    }
}

impl<T> Drop for OwnedWdfObject<T> {
    fn drop(&mut self) {
        // SAFETY: We're calling the function with a guaranteed valid handle, and the rest is set to
        // sane/null defaults.
        unsafe { object_dereference_actual(self.raw.raw_obj(), null_mut(), 0, null_mut()) }
    }
}

pub trait AsWdfReference: Sealed {
    type ObjectType: 'static;
    fn as_wdf_ref(&self) -> WdfObjectReference<'_, Self::ObjectType>;
}

impl<T> AsWdfReference for OwnedWdfObject<T> {
    type ObjectType = T;

    fn as_wdf_ref(&self) -> WdfObjectReference<'_, Self::ObjectType> {
        self.as_ref()
    }
}
