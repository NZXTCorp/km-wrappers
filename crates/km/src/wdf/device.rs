use super::{
    ffi,
    io_queue::{IoQueue, IoQueueConfig},
    object_attributes::ObjectAttributes,
    AsWdfReference, OwnedWdfObject, RawWdfDevice, WdfObjectReference,
};
use crate::{AsRawMutPtr, Sealed};
use core::ptr::null_mut;
use km_shared::{
    ntstatus::{NtStatus, NtStatusError},
    strings::UnicodeString,
};
use km_sys::{WDFQUEUE, WDF_OBJECT_ATTRIBUTES};

/// A guaranteed valid [`WDFDEVICE`](km_sys::WDFDEVICE).
///
/// Note that the system will reject I/O requests to the device until it is
/// [initialized](`DeviceNonInitialized`).
#[repr(transparent)]
#[derive(Clone)]
pub struct Device(pub(crate) OwnedWdfObject<RawWdfDevice>);
impl Sealed for Device {}

impl AsWdfReference for Device {
    type ObjectType = RawWdfDevice;

    fn as_wdf_ref(&self) -> WdfObjectReference<'_, Self::ObjectType> {
        self.0.as_wdf_ref()
    }
}

impl Device {
    /// Builds a new `Device`.
    ///
    /// The system will only start sending I/O requests to the device after it is initialized (see
    /// [`DeviceNonInitialized`]).
    ///
    /// ## Safety
    /// The caller is responsible for ensuring that `handle` is a valid
    /// [`WDFDEVICE`](km_sys::WDFDEVICE).
    pub(crate) unsafe fn new(handle: OwnedWdfObject<RawWdfDevice>) -> Self {
        Self(handle)
    }

    pub fn create_symbolic_link(
        &mut self,
        symbolic_link_name: &UnicodeString,
    ) -> Result<NtStatus, NtStatusError> {
        // SAFETY: The wrapped `WDFDEVICE` is guaranteed to be valid, and `symbolic_link_name` is
        // guaranteed to be a valid pointer. `create_symbolic_link` can also be called multiple
        // times.
        unsafe { ffi::device_create_symbolic_link(self.as_wdf_ref(), symbolic_link_name) }.result()
    }

    pub fn create_io_queue(
        &mut self,
        config: &mut IoQueueConfig,
        mut queue_attributes: Option<&mut ObjectAttributes>,
    ) -> Result<IoQueue, NtStatusError> {
        let mut queue: WDFQUEUE = null_mut();

        // SAFETY: All pointers are guaranteed to be valid.
        unsafe {
            ffi::io_queue_create(
                self.0.as_wdf_ref(),
                &mut config.0,
                queue_attributes
                    .as_raw_mut_ptr()
                    .cast::<WDF_OBJECT_ATTRIBUTES>(),
                &mut queue,
            )
        }
        .result()?;

        debug_assert!(!queue.is_null());

        // SAFETY: `queue` is guaranteed to be valid here.
        Ok(unsafe { IoQueue::new(OwnedWdfObject::from_new_raw(queue)) })
    }
}

pub struct DeviceNonInitialized {
    pub(crate) device: Device,
}

impl DeviceNonInitialized {
    /// Gets the wrapped [`Device`].
    ///
    /// ## Safety
    /// The caller is responsible for ensuring that the usage of the returned is valid while the
    /// device has not [finished initializing](DeviceNonInitialized::finish_initialization).
    pub unsafe fn device(&mut self) -> &mut Device {
        &mut self.device
    }

    /// Finishes the initialization of the device.
    ///
    /// The system stops rejecting I/O requests to the device after this function is called.
    pub fn finish_initialization(self) -> Device {
        // SAFETY: FFI call; the device is guaranteed to be valid.
        unsafe {
            ffi::control_finish_initializing(self.device.as_wdf_ref());
        }

        self.device
    }
}
