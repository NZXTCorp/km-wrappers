use core2::io::Write as _;
use km_shared::{cstrz, ntstatus::NtStatus};
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

        // Buffering this might be an idea, but it currently fails because of
        // https://github.com/technocreatives/core2/issues/12. Also, since kernel stack space is
        // limited, using a stack buffer might be dangerous as well. We're in kernel space, so
        // repeated calls to the API shouldn't be overly expensive either.
        let _ = dbgprint_writer.write_fmt(format_args!("{}\n", *record.args()));
    }

    fn flush(&self) {}
}

struct DbgPrintWriter {
    component: DPFLTR_TYPE,
    level: ULONG,
}

impl core2::io::Write for DbgPrintWriter {
    fn write(&mut self, buf: &[u8]) -> core2::io::Result<usize> {
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
                cstrz!("%s").as_ptr().cast(),
                max_dbgprint_buf.as_ptr(),
            )
        }
        .into()
        {
            NtStatus::STATUS_SUCCESS => Ok(write_len),
            _ => Err(core2::io::Error::new(core2::io::ErrorKind::Other, "")),
        }
    }

    fn flush(&mut self) -> core2::io::Result<()> {
        Ok(())
    }
}
