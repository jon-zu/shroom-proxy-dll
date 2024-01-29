use std::ffi::c_void;

use crate::fn_ref;

pub mod ztl;
pub mod error_codes;
/*pub mod com {
    pub mod iface;
}
pub mod client_socket;
*/

use self::ztl::zxstr::ZXString8;

pub type CLogo = c_void;

pub mod addr {

    pub const CLOGO_INIT: usize = 0x60e240;
    pub const CLOGO_END: usize = 0x60bd00;
    pub const CMSGBOX_INIT: usize = 0x669370;
}

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

fn_ref!(
    cmsgbox_init,
    addr::CMSGBOX_INIT,
    unsafe extern "thiscall" fn(*mut c_void, msg: ZXString8, link: ZXString8, desc: ZXString8)
);
