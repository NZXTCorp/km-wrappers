use km_shared::{strings::make_const_unicode_string, wchz};
use km_sys::UNICODE_STRING;

// from wdmsec.h - copied over instead of referencing the extern static to allow referencing it in
// safe context
pub const SDDL_DEVOBJ_SYS_ALL_ADM_RWX_WORLD_RW_RES_R: UNICODE_STRING = make_const_unicode_string(
    wchz!("D:P(A;;GA;;;SY)(A;;GRGWGX;;;BA)(A;;GRGW;;;WD)(A;;GR;;;RC)"),
);
