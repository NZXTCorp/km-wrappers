use super::WdfObjectReference;
use core::mem::{size_of, transmute, zeroed};
use km_sys::{ULONG, WDFDRIVER__, WDF_DRIVER_CONFIG, WDF_DRIVER_INIT_FLAGS};

pub type WdfDriverUnload = unsafe extern "C" fn(WdfObjectReference<'_, WDFDRIVER__>) -> ();

pub enum DriverConfig {
    Pnp {
        // unimplemented
    },
    NonPnp {
        /// The driver's unload routine.
        ///
        /// This has to be `Some` in order for the driver to be unloadable. See also this [WDK
        /// Sample][WDKSample].
        ///
        /// [WDKSample]: https://github.com/microsoft/Windows-driver-samples/blob/80c104ad0cef2a4fb55aaee7d494f30af5fb44b4/general/ioctl/kmdf/sys/nonpnp.c#L103-L106
        driver_unload: Option<WdfDriverUnload>,
    },
}

impl From<DriverConfig> for WDF_DRIVER_CONFIG {
    fn from(cfg: DriverConfig) -> Self {
        match cfg {
            DriverConfig::Pnp { .. } => unimplemented!("PnP support unimplemented"),
            DriverConfig::NonPnp { driver_unload } => {
                let mut wdf_config = driver_config_init();

                wdf_config.DriverInitFlags =
                    WDF_DRIVER_INIT_FLAGS::WdfDriverInitNonPnpDriver.0 as u32;
                wdf_config.EvtDriverUnload = driver_unload.map(|f| {
                    // SAFETY: `WdfDriverUnload` is FFI-compatible to `WDF_DRIVER_UNLOAD`
                    unsafe { transmute(f) }
                });

                wdf_config
            }
        }
    }
}

#[must_use]
#[inline(always)]
fn driver_config_init() -> WDF_DRIVER_CONFIG {
    // SAFETY: after setting the size the value is valid even for FFI
    unsafe {
        let mut c: WDF_DRIVER_CONFIG = zeroed();
        c.Size = size_of::<WDF_DRIVER_CONFIG>() as ULONG;
        c
    }
}
