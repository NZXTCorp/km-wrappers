use super::{RawWdfDevice, RawWdfFileObject, RawWdfRequest, WdfObjectReference};
use core::mem::{size_of, transmute};
use km_sys::WDF_FILEOBJECT_CONFIG;

pub type EvtDeviceFileCreate = unsafe extern "C" fn(
    device: WdfObjectReference<'_, RawWdfDevice>,
    request: WdfObjectReference<'_, RawWdfRequest>,
    file_object: WdfObjectReference<'_, RawWdfFileObject>,
);

pub struct FileObjectConfig(pub(crate) WDF_FILEOBJECT_CONFIG);

impl FileObjectConfig {
    /// Creates a new `FileObjectConfig` with the default settings
    #[inline(always)]
    pub fn new(init: FileObjectConfigInit) -> Self {
        Self(WDF_FILEOBJECT_CONFIG {
            Size: size_of::<WDF_FILEOBJECT_CONFIG>() as u32,

            EvtDeviceFileCreate: init.evt_device_file_create.map(|f| {
                // SAFETY: The function pointer definition is FFI-compatible.
                unsafe { transmute(f) }
            }),
            EvtFileClose: None,
            EvtFileCleanup: None,
            AutoForwardCleanupClose: km_sys::WDF_TRI_STATE::WdfUseDefault,
            FileObjectClass: km_sys::WDF_FILEOBJECT_CLASS::WdfFileObjectWdfCannotUseFsContexts,
        })
    }
}

pub struct FileObjectConfigInit {
    // the rest will be added on demand
    pub evt_device_file_create: Option<EvtDeviceFileCreate>,
}
