use std::ffi::c_void;

use crate::fn_ref;

use super::{CWvsApp, IWzFileSystem, ZtlBStrT};

pub type IResMan = c_void;
pub type IWzNameSpace = c_void;
    //pub type IWzFileSystem = c_void;

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ResManParam: u32 {
        const AUTO_SERIALIZE = 1;
        const AUTO_SERIALIZE_NO_CACHE = 2;
        const NO_AUTO_SERIALIZE = 4;
        const AUTO_REPARSE = 0x10;
        const NO_AUTO_REPARSE = 0x20;
        const AUTO_REPARSE_MASK = 0x30;
        const DEFAULT_AUTO_SERIALIZE = 0;
        const DEFAULT_AUTO_REPARSE = 0;
        const RC_AUTO_SERIALIZE_MASK =  (Self::AUTO_SERIALIZE.bits() | Self::AUTO_SERIALIZE_NO_CACHE.bits() | Self::NO_AUTO_SERIALIZE.bits());
    }
}

fn_ref!(
    iwz_res_man_set_param,
    0x9c0920,
    // this, nParam, nRetaintime, nNameSpaceCacheTime
    unsafe extern "thiscall" fn(*const IResMan, ResManParam, i32, i32)
);

fn_ref!(
    iwz_namespace_mount,
    0x9c8db0,
    // this, sPath, pDown, nPriority
    unsafe extern "thiscall" fn(*const IWzNameSpace, ZtlBStrT, *const IWzNameSpace, i32)
);

fn_ref!(
    iwz_filesystem_init,
    0x9c8e40,
    // this, sPath
    unsafe extern "thiscall" fn(*const IWzFileSystem, ZtlBStrT)
);

fn_ref!(
    cwvs_app_init_res_man,
    0x009c9540,
    unsafe extern "thiscall" fn(*const CWvsApp)
);