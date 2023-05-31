use super::{
    device::Device, ffi, AsWdfReference, OwnedWdfObject, RawWdfQueue, RawWdfRequest,
    WdfObjectReference,
};
use crate::private::Sealed;
use core::{
    intrinsics::transmute,
    mem::{size_of, zeroed},
};
use km_shared::ioctl::IoControlCode;
use km_sys::{ULONG, WDF_IO_QUEUE_CONFIG, WDF_IO_QUEUE_DISPATCH_TYPE, WDF_TRI_STATE};

pub type IoQueueDispatchType = WDF_IO_QUEUE_DISPATCH_TYPE;

pub enum IoQueueConfigInit {
    Pnp {
        // unimplemented
    },
    NonPnp {
        dispatch_type: IoQueueDispatchType,
        evt_io_device_control: Option<EvtIoDeviceControl>,
    },
}

impl IoQueueConfigInit {
    /// Builds the I/O queue config.
    ///
    /// ## Safety
    ///
    /// The caller ensures that the right enum variant is used for the right driver type, and that
    /// `EvtIoStop` is set for non-PNP queues if needed (see notes below).
    ///
    /// ## Notes
    ///
    /// Relevant comment from WDK sample:
    ///
    /// > By default, Static Driver Verifier (SDV) displays a warning if it
    /// > doesn't find the EvtIoStop callback on a power-managed queue.
    /// > The 'assume' below causes SDV to suppress this warning. If the driver
    /// > has not explicitly set PowerManaged to WdfFalse, the framework creates
    /// > power-managed queues when the device is not a filter driver.  Normally
    /// > the EvtIoStop is required for power-managed queues, but for this driver
    /// > it is not needed b/c the driver doesn't hold on to the requests or
    /// > forward them to other drivers. This driver completes the requests
    /// > directly in the queue's handlers. If the EvtIoStop callback is not
    /// > implemented, the framework waits for all driver-owned requests to be
    /// > done before moving in the Dx/sleep states or before removing the
    /// > device, which is the correct behavior for this type of driver.
    /// > If the requests were taking an indeterminate amount of time to complete,
    /// > or if the driver forwarded the requests to a lower driver/another stack,
    /// > the queue should have an EvtIoStop/EvtIoResume.
    #[must_use]
    pub unsafe fn build(self) -> IoQueueConfig {
        match self {
            IoQueueConfigInit::Pnp { .. } => unimplemented!("PnP support unimplemented"),
            IoQueueConfigInit::NonPnp {
                dispatch_type,
                evt_io_device_control,
            } => {
                let mut config = IoQueueConfig::init_default_queue(dispatch_type);

                config.0.EvtIoDeviceControl =
                    // SAFETY: `EvtIoDeviceControl` is defined to be compatible to
                    // `PFN_WDF_IO_QUEUE_IO_DEVICE_CONTROL` by using repr(transparent) wrappers.
                    evt_io_device_control.map(|f| unsafe { transmute(f) });

                config
            }
        }
    }
}

pub struct IoQueueConfig(pub(crate) WDF_IO_QUEUE_CONFIG);

impl IoQueueConfig {
    #[must_use]
    fn init_default_queue(dispatch_type: IoQueueDispatchType) -> Self {
        // SAFETY: It is initialized the same way as the force-inlined fn
        // `WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE` of the WDF would
        let config = unsafe {
            let mut config: WDF_IO_QUEUE_CONFIG = zeroed();
            config.Size = size_of::<WDF_IO_QUEUE_CONFIG>() as ULONG;

            config.PowerManaged = WDF_TRI_STATE::WdfUseDefault;
            config.DefaultQueue = true as _;
            config.DispatchType = dispatch_type;

            if config.DispatchType == IoQueueDispatchType::WdfIoQueueDispatchParallel {
                config.Settings.Parallel.NumberOfPresentedRequests = -1i32 as ULONG;
            }

            config
        };

        IoQueueConfig(config)
    }
}

pub type EvtIoDeviceControl = unsafe extern "C" fn(
    WdfObjectReference<'_, RawWdfQueue>,   // Queue
    WdfObjectReference<'_, RawWdfRequest>, // Request
    usize,                                 // OutputBufferLength
    usize,                                 // InputBufferLength
    IoControlCode,                         // IoControlCode
);

#[derive(Debug, Clone)]
pub struct IoQueue(OwnedWdfObject<RawWdfQueue>);
impl Sealed for IoQueue {}

impl IoQueue {
    /// Builds a new `Device`.
    ///
    /// The system will only start sending I/O requests to the device after it is initialized (see
    /// [`DeviceNonInitialized`]).
    ///
    /// ## Safety
    /// The caller is responsible for ensuring that `handle` is a valid
    /// [`WDFQUEUE`](km_sys::WDFQUEUE).
    pub(crate) unsafe fn new(handle: OwnedWdfObject<RawWdfQueue>) -> Self {
        Self(handle)
    }
}

impl AsWdfReference for IoQueue {
    type ObjectType = RawWdfQueue;

    fn as_wdf_ref(&self) -> super::WdfObjectReference<'_, Self::ObjectType> {
        self.0.as_wdf_ref()
    }
}

impl From<OwnedWdfObject<RawWdfQueue>> for IoQueue {
    fn from(owned: OwnedWdfObject<RawWdfQueue>) -> Self {
        Self(owned)
    }
}

impl IoQueue {
    pub fn device(&self) -> Device {
        // SAFETY: The queue is guaranteed to be valid.
        unsafe { Device::new(ffi::io_queue_get_device(self.0.as_wdf_ref()).to_owned()) }
    }
}
