use embedded_io::Write as _;
use km_shared::ntstatus::NtStatus;
use km_sys::{
    DbgPrintEx, DPFLTR_ERROR_LEVEL, DPFLTR_INFO_LEVEL, DPFLTR_TRACE_LEVEL, DPFLTR_TYPE,
    DPFLTR_WARNING_LEVEL, ULONG, _DPFLTR_TYPE,
};
use log::Log;

pub struct KernelLogger;

impl Log for KernelLogger {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        let mut dbgprint_writer = DbgPrintWriter {
            component: _DPFLTR_TYPE::DPFLTR_IHVDRIVER_ID,
            level: match record.level() {
                log::Level::Error => DPFLTR_ERROR_LEVEL,
                log::Level::Warn => DPFLTR_WARNING_LEVEL,
                log::Level::Info => DPFLTR_INFO_LEVEL,
                log::Level::Trace => DPFLTR_TRACE_LEVEL,
                // debug is not inherently supported by `DPFLTR` constants, fall back to trace level
                log::Level::Debug => DPFLTR_TRACE_LEVEL,
            },
        };

        let _ = writeln!(dbgprint_writer, "{}", *record.args());
    }

    fn flush(&self) {}
}

struct DbgPrintWriter {
    component: DPFLTR_TYPE,
    level: ULONG,
}

impl embedded_io::ErrorType for DbgPrintWriter {
    type Error = embedded_io::ErrorKind;
}

impl embedded_io::Write for DbgPrintWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, embedded_io::ErrorKind> {
        // DbgPrintEx/KdPrintEx only transmit 512 bytes at a time, so we only transfer
        // 511 bytes plus null terminator per call.

        if buf.is_empty() {
            return Ok(0);
        }

        const MAX_DBGPRINT_BUF_LEN: usize = 512;
        const MAX_DBGPRINT_BUF_LEN_WITHOUT_NUL: usize = MAX_DBGPRINT_BUF_LEN - 1;

        let mut max_dbgprint_buf = [0u8; MAX_DBGPRINT_BUF_LEN];

        let write_len = usize::min(MAX_DBGPRINT_BUF_LEN_WITHOUT_NUL, buf.len());
        debug_assert!(write_len < max_dbgprint_buf.len());
        debug_assert!(write_len <= buf.len());
        max_dbgprint_buf[..write_len].copy_from_slice(&buf[..write_len]);

        // SAFETY:
        // - `component` is one of the valid `DPFLTR_TYPE` constants
        // - `level` is one of the `DPFLTR_*_LEVEL` constants
        // - the format string is valid and zero-terminated
        // - the fourth parameter matches the format specifier in the format string, and is both
        //   short enough that nothing will be cut off, and zero-terminated
        match unsafe {
            DbgPrintEx(
                self.component.0 as ULONG,
                self.level,
                c"%s".as_ptr().cast(),
                max_dbgprint_buf.as_ptr(),
            )
        }
        .into()
        {
            NtStatus::STATUS_SUCCESS => Ok(write_len),
            _ => Err(embedded_io::ErrorKind::Other),
        }
    }

    fn flush(&mut self) -> Result<(), embedded_io::ErrorKind> {
        Ok(())
    }
}
