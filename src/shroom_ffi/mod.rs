use std::ffi::c_void;

use crate::fn_ref;

pub mod addr;

pub type CLogo = c_void;

fn_ref!(
    clogo_init,
    addr::CLOGO_INIT,
    unsafe extern "thiscall" fn(*mut CLogo, param: *const c_void)
);

fn_ref!(
    clogo_end,
    addr::CLOGO_END,
    unsafe extern "thiscall" fn(*mut CLogo)
);