use super::{ffi, AsWdfReference, OwnedWdfObject, RawWdfRequest};
use crate::{mode::ProcessorMode, private::Sealed};
use bytemuck::{checked::CheckedCastError, CheckedBitPattern, NoUninit};
use core::{
    cell::Cell,
    mem::size_of,
    ops::{Deref, DerefMut},
    ptr::null_mut,
    slice,
};
use km_shared::{
    ioctl::TypedIoControlCode,
    ntstatus::{NtStatus, NtStatusError},
};
use snafu::{ensure, ResultExt, Snafu};

/// A high-level wrapper around a [`RawRequest`](raw I/O control request).
// (intentionally not providing a `Clone` impl as we are guaranteeing unique access to the buffers)
pub struct Request {
    obj: OwnedWdfObject<RawWdfRequest>,
    /// Flag for manual borrow checking of the output buffer.
    output_buffer_borrowed: Cell<bool>,
}
impl Sealed for Request {}

impl AsWdfReference for Request {
    type ObjectType = RawWdfRequest;

    fn as_wdf_ref(&self) -> super::WdfObjectReference<'_, Self::ObjectType> {
        self.obj.as_wdf_ref()
    }
}

impl From<OwnedWdfObject<RawWdfRequest>> for Request {
    fn from(obj: OwnedWdfObject<RawWdfRequest>) -> Self {
        Self {
            obj,
            output_buffer_borrowed: Cell::new(false),
        }
    }
}

#[derive(Debug, Snafu)]
pub enum IoCtlError {
    OutputBufferAlreadyBorrowed,
    #[snafu(context(false))]
    NtStatus {
        source: NtStatusError,
    },
    Cast {
        output_buffer: bool,
        inner: CheckedCastError,
    },
}

impl Request {
    /// Retrieve typed buffers for an I/O control request and calls the provided closure to handle
    /// the request.
    ///
    /// # Safety
    /// Since this function gives access to the output buffer, the same requirements as
    /// [`Self::retrieve_output_buffer`] apply.
    pub unsafe fn handle_ioctl<I, O, R>(
        &self,
        // just to get the types without needing to manually specify them
        _ioctl: TypedIoControlCode<I, O>,
        f: impl FnOnce(&I, &mut O) -> R,
    ) -> Result<R, IoCtlError>
    where
        I: CheckedBitPattern,
        O: NoUninit + CheckedBitPattern,
    {
        let input_buffer = if size_of::<I>() > 0 {
            self.retrieve_input_buffer(size_of::<I>())?
        } else {
            InputBuffer {
                slice: &[] as &'static [u8],
            }
        };

        let input = bytemuck::checked::try_from_bytes(&input_buffer).map_err(|e| {
            CastSnafu {
                output_buffer: false,
                inner: e,
            }
            .build()
        })?;

        let mut output_buffer = if size_of::<O>() > 0 {
            // SAFETY: The requirements for this are promised to be upheld by the caller.
            unsafe { self.retrieve_output_buffer(size_of::<O>()) }.map_err(|e| match e {
                RetrieveOutputBufferError::OutputBufferAlreadyBorrowed => {
                    IoCtlError::OutputBufferAlreadyBorrowed
                }
                RetrieveOutputBufferError::NtStatus { source } => IoCtlError::NtStatus { source },
            })?
        } else {
            OutputBuffer {
                request: self,
                slice: &mut [] as &'static mut [u8],
            }
        };

        let output = bytemuck::checked::try_from_bytes_mut(&mut output_buffer).map_err(|e| {
            CastSnafu {
                output_buffer: true,
                inner: e,
            }
            .build()
        })?;

        let r = f(input, output);

        if size_of::<O>() > 0 {
            self.set_information(size_of::<O>() as u64);
        }

        Ok(r)
    }

    // Retrieves the input buffer of the request as a borrowed slice.
    ///
    /// See [MSDN] for more details on the underlying function.
    ///
    /// [MSDN]: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfrequest/nf-wdfrequest-wdfrequestretrieveinputbuffer
    pub fn retrieve_input_buffer(
        &self,
        minimum_required_length: usize,
    ) -> Result<InputBuffer<'_>, NtStatusError> {
        let mut buffer = null_mut();
        let mut buffer_len = 0;

        // SAFETY: We call the function with all valid parameters.
        unsafe {
            ffi::request_retrieve_input_buffer(
                self.obj.as_wdf_ref(),
                minimum_required_length,
                &mut buffer,
                &mut buffer_len,
            )
            .result()?;
        }

        Ok(InputBuffer {
            // SAFETY: We trust the kernel to give us valid data when the FFI call was successful.
            slice: unsafe { slice::from_raw_parts(buffer.cast(), buffer_len) },
        })
    }

    /// Retrieves the output buffer of the request as a borrowed mutable slice.
    ///
    /// Because the output buffer may be mutated, this function ensures that the loan must be
    /// returned before requesting it again, or this function will fail with a
    /// [`RetrieveOutputBufferError::OutputBufferAlreadyBorrowed`] error.
    ///
    /// See [MSDN] for more details on the underlying function.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is only one `Request` accessing the output buffer. It *is*
    /// validated within one instance of `Request` (see above), but not across multiple instances.
    ///
    /// [MSDN]: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfrequest/nf-wdfrequest-wdfrequestretrieveoutputbuffer
    pub unsafe fn retrieve_output_buffer(
        &self,
        minimum_required_length: usize,
    ) -> Result<OutputBuffer<'_>, RetrieveOutputBufferError> {
        ensure!(
            !self.output_buffer_borrowed.get(),
            retrieve_output_buffer_error::OutputBufferAlreadyBorrowedSnafu
        );

        let mut buffer = null_mut();
        let mut buffer_len = 0;

        // SAFETY: We call the function with all valid parameters.
        unsafe {
            ffi::request_retrieve_output_buffer(
                self.obj.as_wdf_ref(),
                minimum_required_length,
                &mut buffer,
                &mut buffer_len,
            )
            .result()
            .context(retrieve_output_buffer_error::NtStatusSnafu)?;
        }

        // SAFETY: We checked that the output buffer is currently not accessible at the start of this
        // function.
        Ok(unsafe { OutputBuffer::new(self, buffer.cast(), buffer_len) })
    }

    /// Sets the number of bytes written to the output buffer.
    pub fn set_information(&self, information: u64) {
        // SAFETY: We call the function with all valid parameters.
        unsafe {
            ffi::request_set_information(self.obj.as_wdf_ref(), information);
        }
    }

    pub fn requestor_mode(&self) -> ProcessorMode {
        // SAFETY: We call the ffi function with all valid parameters. `WdfRequestGetRequestorMode`
        // always returns a valid mode.
        unsafe {
            ProcessorMode::from_kprocessor_mode_unchecked(ffi::request_get_requestor_mode(
                self.obj.as_wdf_ref(),
            ))
        }
    }

    /// Completes the I/O request.
    ///
    /// This *must* be called at some point (to not have the caller be stuck forever), but not
    /// necessarily in the [I/O Control handler][ioctl] itself.
    ///
    /// See [MSDN] for more details on the underlying function.
    ///
    /// [ioctl]: super::io_queue::EvtIoDeviceControl
    /// [MSDN]: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfrequest/nf-wdfrequest-wdfrequestcomplete
    pub fn complete(self, status: NtStatus) {
        // SAFETY: `self.0` is guaranteed to be a valid pointer to a `WDFREQUEST`
        unsafe { ffi::request_complete(self.obj.as_wdf_ref(), status) }
    }
}

/// An input buffer returned from [`Request::retrieve_input_buffer`].
pub struct InputBuffer<'a> {
    slice: &'a [u8],
}

impl Deref for InputBuffer<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.slice
    }
}

/// An output buffer returned from [`Request::retrieve_output_buffer`].
pub struct OutputBuffer<'a> {
    request: &'a Request,
    slice: &'a mut [u8],
}

impl<'a> OutputBuffer<'a> {
    /// # Safety
    /// The caller must ensure that the output buffer is not currently borrowed or otherwise
    /// accessible.
    unsafe fn new(request: &'a Request, buffer: *mut u8, buffer_len: usize) -> Self {
        debug_assert!(!request.output_buffer_borrowed.get());

        // We do manual borrow checking here, as we wouldn't be able to ensure uniqueness for the
        // &mut slice we want to return otherwise. See the `Drop` impl where this gets set back to
        // `false`.
        request.output_buffer_borrowed.set(true);

        OutputBuffer {
            request,
            // SAFETY:
            // - We trust the kernel to give us valid data when the FFI call was successful.
            // - The caller asserts that the buffer is not currently borrowed or otherwise
            //   accessible.
            slice: unsafe { slice::from_raw_parts_mut(buffer, buffer_len) },
        }
    }
}

impl Deref for OutputBuffer<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.slice
    }
}

impl DerefMut for OutputBuffer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.slice
    }
}

impl Drop for OutputBuffer<'_> {
    fn drop(&mut self) {
        // See `Self::new` for why we do this manually (or at all).
        self.request.output_buffer_borrowed.set(false);
    }
}

/// An error returned from [`Request::retrieve_output_buffer`].
#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum RetrieveOutputBufferError {
    OutputBufferAlreadyBorrowed,
    NtStatus { source: NtStatusError },
}
