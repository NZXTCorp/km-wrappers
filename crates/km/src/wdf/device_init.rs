use super::{
    device::{Device, DeviceNonInitialized},
    ffi,
    file_object::FileObjectConfig,
    object_attributes::ObjectAttributes,
    DeviceIoType, OwnedWdfObject,
};
use crate::{AsRawMutPtr, AsRawPtr};
use core::ptr::{null_mut, NonNull};
use km_shared::{
    ntstatus::{NtStatus, NtStatusError},
    strings::UnicodeString,
};
use km_sys::{BOOLEAN, WDFDEVICE, WDFDEVICE_INIT, WDF_OBJECT_ATTRIBUTES};

pub struct DeviceInit(pub(crate) NonNull<WDFDEVICE_INIT>);

impl Drop for DeviceInit {
    fn drop(&mut self) {
        // SAFETY: A `DeviceInit` is guaranteed to contain a valid pointer to a `WDFDEVICE_INIT`
        unsafe { Self::free_raw(self.0) };
    }
}

impl DeviceInit {
    /// Frees a raw [`WDFDEVICE_INIT`].
    ///
    /// ## Safety
    ///
    /// The caller is responsible for ensuring that the pointer is pointing to a
    /// [`WDFDEVICE_INIT`].
    unsafe fn free_raw(ptr: NonNull<WDFDEVICE_INIT>) {
        // SAFETY: see this function's documentation.
        unsafe {
            ffi::device_init_free(ptr.as_ptr());
        }
    }

    /// Builds a new `DeviceInit` from a raw [`WDFDEVICE_INIT`].
    ///
    /// ## Safety
    ///
    /// The caller is responsible for ensuring that the pointer
    /// - is pointing to a [`WDFDEVICE_INIT`]
    /// - is not already owned by another `DeviceInit`
    pub(crate) unsafe fn new(ptr: NonNull<WDFDEVICE_INIT>) -> Self {
        Self(ptr)
    }

    pub fn set_exclusive_access(&mut self, exclusive_access: bool) {
        // SAFETY: A `DeviceInit` is guaranteed to contain a valid pointer to a `WDFDEVICE_INIT`.
        unsafe { ffi::device_init_set_exclusive(self.0.as_ptr(), exclusive_access as BOOLEAN) }
    }

    pub fn set_io_type(&mut self, io_type: DeviceIoType) {
        // SAFETY: A `DeviceInit` is guaranteed to contain a valid pointer to a `WDFDEVICE_INIT`.
        // `DeviceIoType` limits to valid values for this parameter.
        unsafe { ffi::device_init_set_io_type(self.0.as_ptr(), io_type) }
    }

    pub fn assign_name(
        &mut self,
        device_name: Option<&UnicodeString>,
    ) -> Result<NtStatus, NtStatusError> {
        let unicode_ptr = device_name.as_raw_ptr();

        // SAFETY:
        // - A `DeviceInit` is guaranteed to contain a valid pointer to a `WDFDEVICE_INIT`.
        // - `unicode_ptr` is guaranteed to be either `null_ptr` or pointing to a valid value.
        unsafe { ffi::device_init_assign_name(self.0.as_ptr(), unicode_ptr) }.result()
    }

    pub fn set_file_object_config(
        &mut self,
        mut file_object_config: FileObjectConfig,
        mut file_object_attributes: Option<&mut ObjectAttributes>,
    ) {
        // SAFETY: The ffi call happens with guaranteed correct parameters.
        unsafe {
            ffi::device_init_set_file_object_config(
                self.0.as_ptr(),
                &mut file_object_config.0,
                file_object_attributes
                    .as_raw_mut_ptr()
                    .cast::<WDF_OBJECT_ATTRIBUTES>(),
            )
        }
    }

    pub fn create_device(
        self,
        mut device_attributes: Option<&mut ObjectAttributes>,
    ) -> Result<DeviceNonInitialized, NtStatusError> {
        // WdfDeviceCreate deallocates our wrapped `WDFDEVICE_INIT` automatically on success,
        // setting the pointer to null, which would be UB for our `DeviceInit` containing a
        // guaranteed valid non-null pointer to a `WDFDEVICE_INIT`.
        let mut device_init_ptr = {
            let device_init = self.0.as_ptr();
            // prevent the `DeviceInit` from being drop-handled
            core::mem::forget(self);
            device_init
        };

        let obj_attr_ptr = device_attributes
            .as_raw_mut_ptr()
            // `ObjectAttributes` is a repr-transparent wrapper around
            // `WDF_OBJECT_ATTRIBUTES`.
            .cast::<WDF_OBJECT_ATTRIBUTES>();

        let mut device: WDFDEVICE = null_mut();

        let result =
        // SAFETY:
        // - `device_init_ptr` is guaranteed to be a valid pointer to a `WDFDEVICE_INIT`.
        // - `device` is an out parameter.
            unsafe { ffi::device_create(&mut device_init_ptr, obj_attr_ptr, &mut device) }.result();

        match result {
            Ok(_) => {
                let device = OwnedWdfObject::from_new_raw(device);
                Ok(DeviceNonInitialized {
                    // SAFETY: Guaranteed to be a valid pointer to a `WDFDEVICE` since
                    // `ffi::device_create` succeeded.
                    device: unsafe { Device::new(device) },
                })

                // device_init must *not* be freed in the success case:
                // > Your driver must not call WdfDeviceInitFree after a successful call to
                // > WdfDeviceCreate.
            }
            Err(e) => {
                // check if the pointer is not null, and if so, free the memory
                if let Some(device_init) = NonNull::new(device_init_ptr) {
                    // SAFETY: The `DeviceInit` is guaranteed to be valid, so we can safely call
                    // `free_raw`.
                    unsafe { Self::free_raw(device_init) };
                }

                Err(e)
            }
        }
    }
}
