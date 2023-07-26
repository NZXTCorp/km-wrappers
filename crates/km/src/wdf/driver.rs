use super::{
    device_init::DeviceInit, driver_config::DriverConfig, ffi, object_attributes::ObjectAttributes,
    AsWdfReference, OwnedWdfObject, RawWdfDriver, WdfObjectReference,
};
use crate::{AsRawMutPtr, DriverObjectHandle, Sealed, UnicodeStringHandle};
use core::ptr::{null_mut, NonNull};
use km_shared::{ntstatus::NtStatusError, strings::UnicodeString};
use km_sys::{WDFDRIVER, WDF_OBJECT_ATTRIBUTES};

#[repr(transparent)]
#[derive(Clone)]
pub struct Driver(OwnedWdfObject<RawWdfDriver>);
impl Sealed for Driver {}

impl From<WdfObjectReference<'_, RawWdfDriver>> for Driver {
    fn from(raw: WdfObjectReference<'_, RawWdfDriver>) -> Self {
        Self(raw.to_owned())
    }
}

impl AsWdfReference for Driver {
    type ObjectType = RawWdfDriver;

    fn as_wdf_ref(&self) -> WdfObjectReference<'_, Self::ObjectType> {
        self.0.as_wdf_ref()
    }
}

impl Driver {
    // we need the mutable ptr `driver_object` and `registry_path`
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn create(
        driver_object: &mut DriverObjectHandle,
        registry_path: &mut UnicodeStringHandle,
        mut driver_attributes: Option<&mut ObjectAttributes>,
        driver_config: DriverConfig,
    ) -> Result<Driver, NtStatusError> {
        let mut driver: WDFDRIVER = null_mut();
        // SAFETY: We're calling `driver_create` with guaranteed valid values.
        unsafe {
            ffi::driver_create(
                driver_object.0,
                registry_path.0,
                driver_attributes
                    .as_raw_mut_ptr()
                    .cast::<WDF_OBJECT_ATTRIBUTES>(),
                &mut driver_config.into(),
                &mut driver,
            )
        }
        .result()?;

        debug_assert!(!driver.is_null());

        Ok(Driver(OwnedWdfObject::from_new_raw(driver)))
    }

    pub fn allocate_control_device_init(&mut self, sddl: &UnicodeString) -> Option<DeviceInit> {
        // SAFETY: sddl is a guaranteed valid pointer to a UnicodeString
        NonNull::new(unsafe { ffi::control_device_init_allocate(self.as_wdf_ref().raw(), sddl) })
            .map(|ptr| {
                // SAFETY: the `WDFDEVICE_INIT` we get is guaranteed to be valid and doesn't belong to
                // any other `DeviceInit`, satifying the safety contract.
                unsafe { DeviceInit::new(ptr) }
            })
    }
}
