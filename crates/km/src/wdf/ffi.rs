use crate::wdf::{RawWdfObject, WdfObjectReference};
use km_shared::ntstatus::NtStatus;
use km_sys::{
    BOOLEAN, HANDLE, KPROCESSOR_MODE, LONG, PCHAR, PCUNICODE_STRING,
    PCWDF_OBJECT_CONTEXT_TYPE_INFO, PDRIVER_OBJECT, PFN_WDFCONTROLDEVICEINITALLOCATE,
    PFN_WDFCONTROLFINISHINITIALIZING, PFN_WDFDEVICECREATE, PFN_WDFDEVICECREATESYMBOLICLINK,
    PFN_WDFDEVICEINITASSIGNNAME, PFN_WDFDEVICEINITFREE, PFN_WDFDEVICEINITSETEXCLUSIVE,
    PFN_WDFDEVICEINITSETFILEOBJECTCONFIG, PFN_WDFDEVICEINITSETIOTYPE, PFN_WDFDRIVERCREATE,
    PFN_WDFIOQUEUECREATE, PFN_WDFIOQUEUEGETDEVICE, PFN_WDFOBJECTDEREFERENCEACTUAL,
    PFN_WDFOBJECTGETTYPEDCONTEXTWORKER, PFN_WDFOBJECTREFERENCEACTUAL, PFN_WDFREQUESTCOMPLETE,
    PFN_WDFREQUESTGETREQUESTORMODE, PFN_WDFREQUESTRETRIEVEINPUTBUFFER,
    PFN_WDFREQUESTRETRIEVEOUTPUTBUFFER, PFN_WDFREQUESTSETINFORMATION, PVOID, PWDFDEVICE_INIT,
    PWDF_DRIVER_CONFIG, PWDF_DRIVER_GLOBALS, PWDF_FILEOBJECT_CONFIG, PWDF_IO_QUEUE_CONFIG,
    PWDF_OBJECT_ATTRIBUTES, ULONG_PTR, WDFDEVICE, WDFDEVICE__, WDFDRIVER, WDFFUNCENUM, WDFQUEUE,
    WDFQUEUE__, WDFREQUEST__, WDF_DEVICE_IO_TYPE,
};

trait Inner {
    type Inner;
}

impl<T> Inner for Option<T> {
    // Helper impl to get the wrapped type.
    type Inner = T;
}

/// Helper macro to declare a WDF function the way the C macros do.
macro_rules! wdf_function {
    {
        ($fp_ptr:ty, $index:expr):
        $(#[$meta:meta])*
        pub unsafe fn $symbol:ident($($argname:ident: $argtype:ty),* $(,)?) -> $rettype:ty
    } => {
        $(#[$meta])*
        #[inline(always)]
        // needed as the comments below seem to be stripped
        // #[allow(clippy::undocumented_unsafe_blocks)]
        pub unsafe fn $symbol($($argname: $argtype),*) -> $rettype {
            type Ty = unsafe extern "C" fn(PWDF_DRIVER_GLOBALS, $($argtype),*) -> $rettype;

            // SAFETY: We assume here that `$argname`, `$argtype`, and `$rettype` really do
            // correspond to a symbol with the associated type in the `WdfFunctions` function table
            // we're accessing here.
            let fp: *const <$fp_ptr as Inner>::Inner = unsafe {
                core::mem::transmute(
                    ::km_sys::WdfFunctions_01015
                        .offset($index.0 as isize),
                )
            };

            // SAFETY: Trusting that the definition is correct/ffi-compatible.
            let fp: *const Ty = unsafe {
                #[allow(clippy::useless_transmute)]
                core::mem::transmute(fp)
            };

            // SAFETY: We assume that:
            // 1. `fp` is usable as described above, and
            // 2. any invariants for this specific function are upheld by calling code.
            unsafe {
                    (*fp)(::km_sys::WdfDriverGlobals, $($argname),*)
            }
        }
    };
}

wdf_function! {
    (PFN_WDFDRIVERCREATE, WDFFUNCENUM::WdfDriverCreateTableIndex):
    #[must_use]
    pub unsafe fn driver_create(
        driver_object: PDRIVER_OBJECT,
        registry_path: PCUNICODE_STRING,
        driver_attributes: PWDF_OBJECT_ATTRIBUTES,
        driver_config: PWDF_DRIVER_CONFIG,
        driver: *mut WDFDRIVER
    ) -> NtStatus
}

wdf_function! {
    (PFN_WDFCONTROLDEVICEINITALLOCATE, WDFFUNCENUM::WdfControlDeviceInitAllocateTableIndex):
    #[must_use]
    pub unsafe fn control_device_init_allocate(
        driver: WDFDRIVER,
        sddl_string: PCUNICODE_STRING
    ) -> PWDFDEVICE_INIT
}

wdf_function! {
    (PFN_WDFDEVICEINITFREE, WDFFUNCENUM::WdfDeviceInitFreeTableIndex):
    pub unsafe fn device_init_free(
        device_init: PWDFDEVICE_INIT
    ) -> ()
}

wdf_function! {
    (PFN_WDFDEVICEINITSETEXCLUSIVE, WDFFUNCENUM::WdfDeviceInitSetExclusiveTableIndex):
    pub unsafe fn device_init_set_exclusive(
        device_init: PWDFDEVICE_INIT,
        is_exclusive: BOOLEAN
    ) -> ()
}

wdf_function! {
    (PFN_WDFDEVICEINITSETIOTYPE, WDFFUNCENUM::WdfDeviceInitSetIoTypeTableIndex):
    pub unsafe fn device_init_set_io_type(
        device_init: PWDFDEVICE_INIT,
        io_type: WDF_DEVICE_IO_TYPE
    ) -> ()
}

wdf_function! {
    (PFN_WDFDEVICEINITASSIGNNAME, WDFFUNCENUM::WdfDeviceInitAssignNameTableIndex):
    #[must_use]
    pub unsafe fn device_init_assign_name(
        device_init: PWDFDEVICE_INIT,
        device_name: PCUNICODE_STRING
    ) -> NtStatus
}

wdf_function! {
    (PFN_WDFDEVICECREATE, WDFFUNCENUM::WdfDeviceCreateTableIndex):
    #[must_use]
    pub unsafe fn device_create(
        device_init: *mut PWDFDEVICE_INIT,
        device_attributes: PWDF_OBJECT_ATTRIBUTES,
        device: *mut WDFDEVICE
    ) -> NtStatus
}

wdf_function! {
    (PFN_WDFDEVICECREATESYMBOLICLINK, WDFFUNCENUM::WdfDeviceCreateSymbolicLinkTableIndex):
    #[must_use]
    pub unsafe fn device_create_symbolic_link(
        device: WdfObjectReference<'_, WDFDEVICE__>,
        symbolic_link_name: PCUNICODE_STRING
    ) -> NtStatus
}

wdf_function! {
    (PFN_WDFCONTROLFINISHINITIALIZING, WDFFUNCENUM::WdfControlFinishInitializingTableIndex):
    pub unsafe fn control_finish_initializing(
        device: WdfObjectReference<'_, WDFDEVICE__>
    ) -> ()
}

wdf_function! {
    (PFN_WDFIOQUEUECREATE, WDFFUNCENUM::WdfIoQueueCreateTableIndex):
    #[must_use]
    pub unsafe fn io_queue_create(
        device: WdfObjectReference<'_, WDFDEVICE__>,
        config: PWDF_IO_QUEUE_CONFIG,
        queue_attributes: PWDF_OBJECT_ATTRIBUTES,
        queue: *mut WDFQUEUE
    ) -> NtStatus
}

wdf_function! {
    (PFN_WDFREQUESTCOMPLETE, WDFFUNCENUM::WdfRequestCompleteTableIndex):
    pub unsafe fn request_complete(
        request: WdfObjectReference<'_, WDFREQUEST__>,
        status: NtStatus
    ) -> ()
}

wdf_function! {
    (PFN_WDFREQUESTRETRIEVEINPUTBUFFER, WDFFUNCENUM::WdfRequestRetrieveInputBufferTableIndex):
    #[must_use]
    pub unsafe fn request_retrieve_input_buffer(
        request: WdfObjectReference<'_, WDFREQUEST__>,
        minimum_required_length: usize,
        buffer: *mut PVOID,
        length: *mut usize,
    ) -> NtStatus
}

wdf_function! {
    (PFN_WDFREQUESTRETRIEVEOUTPUTBUFFER, WDFFUNCENUM::WdfRequestRetrieveOutputBufferTableIndex):
    #[must_use]
    pub unsafe fn request_retrieve_output_buffer(
        request: WdfObjectReference<'_, WDFREQUEST__>,
        minimum_required_length: usize,
        buffer: *mut PVOID,
        length: *mut usize,
    ) -> NtStatus
}

wdf_function! {
    (PFN_WDFOBJECTGETTYPEDCONTEXTWORKER, WDFFUNCENUM::WdfObjectGetTypedContextWorkerTableIndex):
    #[must_use]
    pub unsafe fn object_get_typed_context_worker(
        handle: WdfObjectReference<'_, RawWdfObject>,
        type_info: PCWDF_OBJECT_CONTEXT_TYPE_INFO,
    ) -> PVOID
}

wdf_function! {
    (PFN_WDFOBJECTREFERENCEACTUAL, WDFFUNCENUM::WdfObjectReferenceActualTableIndex):
    pub unsafe fn object_reference_actual(
        handle: HANDLE,
        tag: PVOID,
        line: LONG,
        file: PCHAR,
    ) -> ()
}

wdf_function! {
    (PFN_WDFOBJECTDEREFERENCEACTUAL, WDFFUNCENUM::WdfObjectDereferenceActualTableIndex):
    pub unsafe fn object_dereference_actual(
        handle: HANDLE,
        tag: PVOID,
        line: LONG,
        file: PCHAR,
    ) -> ()
}

wdf_function! {
    (PFN_WDFIOQUEUEGETDEVICE, WDFFUNCENUM::WdfIoQueueGetDeviceTableIndex):
    pub unsafe fn io_queue_get_device(
        queue: WdfObjectReference<'_, WDFQUEUE__>,
    ) -> WdfObjectReference<'_, WDFDEVICE__>
}

wdf_function! {
    (PFN_WDFREQUESTSETINFORMATION, WDFFUNCENUM::WdfRequestSetInformationTableIndex):
    pub unsafe fn request_set_information(
        request: WdfObjectReference<'_, WDFREQUEST__>,
        information: ULONG_PTR,
    ) -> ()
}

wdf_function! {
    (PFN_WDFREQUESTGETREQUESTORMODE, WDFFUNCENUM::WdfRequestGetRequestorModeTableIndex):
    pub unsafe fn request_get_requestor_mode(
        request: WdfObjectReference<'_, WDFREQUEST__>,
    ) -> KPROCESSOR_MODE
}

wdf_function! {
    (PFN_WDFDEVICEINITSETFILEOBJECTCONFIG, WDFFUNCENUM::WdfDeviceInitSetFileObjectConfigTableIndex):
    pub unsafe fn device_init_set_file_object_config(
        device_init: PWDFDEVICE_INIT,
        file_object_config: PWDF_FILEOBJECT_CONFIG,
        file_object_attributes: PWDF_OBJECT_ATTRIBUTES,
    ) -> ()
}
