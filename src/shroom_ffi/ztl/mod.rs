use std::{
    ffi::{c_int, c_void}, ptr
};

use widestring::{U16CStr, U16CString};
use windows::core::{HRESULT, PCWSTR};

use super::{bstr_assign, bstr_ctor, bstr_free};

pub mod tsec;
pub mod zlist;
pub mod zmap;
pub mod zxarr;
pub mod zxstr;

#[derive(Debug)]
#[repr(C)]
pub struct ZException(pub HRESULT);

#[derive(Debug)]
#[repr(C, packed)]
pub struct ZFatalSection {
    pub tib: *const c_void,
    pub ref_count: isize,
}

pub const ZEXCEPTION_MAGIC: u32 = 0x19930520;

#[derive(Debug)]
#[repr(C, packed)]
pub struct ZRef<T> {
    pub vtable: *const c_void,
    pub ptr: *const T,
}

impl<T> Drop for ZRef<T> {
    fn drop(&mut self) {
        todo!()
    }
}

#[repr(C)]
pub union ZRefNextOrCount<T> {
    pub next: *const ZRef<T>,
    pub count: isize,
}

impl<T> std::fmt::Debug for ZRefNextOrCount<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZRefNextOrCount")
            .field("next/count", unsafe { &self.count })
            .finish()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ZRefCounted<T> {
    pub vtable: *const c_void,
    pub next_or_ref: ZRefNextOrCount<T>,
    pub prev: *const ZRefCounted<T>,
    pub ptr: *const T,
}

#[derive(Debug)]
#[repr(C)]
pub struct TSingleton<T>(pub *mut T);

impl<T> TSingleton<T> {
    pub fn is_instantiated(&self) -> bool {
        !self.0.is_null()
    }

    pub fn get_instance(&self) -> Option<&T> {
        unsafe { self.0.as_ref() }
    }

    pub fn get_instance_mut(&mut self) -> Option<&mut T> {
        unsafe { self.0.as_mut() }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct BStrData {
    pub wstr: *const u16,
    pub str: *const u8,
    pub ref_count: c_int,
}

#[derive(Debug)]
#[repr(C)]
pub struct BStr(pub *mut BStrData);

impl Drop for BStr {
    fn drop(&mut self) {
        unsafe {
            bstr_free()(self as *mut BStr);
        }
    }
}

impl BStr {
    pub fn from_wide(s: PCWSTR) -> Self {
        let mut bstr = Self(ptr::null_mut());
        unsafe {
            bstr_ctor()(&mut bstr as *mut BStr, s);
        }
        bstr
    }

    pub fn from_u16(s: &U16CStr) -> Self {
        Self::from_wide(PCWSTR(s.as_ptr()))
    }

    pub fn from_str(s: &str) -> Self {
        Self::from_u16(&U16CString::from_str(s).unwrap())
    }

    pub fn assign_wide(&mut self, s: PCWSTR) {
        unsafe {
            bstr_assign()(self as *mut BStr, s);
        }
    }

    pub fn as_wide(&self) -> Option<PCWSTR> {
        let data= unsafe { self.0.as_ref()? };
        if !data.wstr.is_null() {
            Some(PCWSTR(data.wstr))
        } else {
            None
        }
    }
}
