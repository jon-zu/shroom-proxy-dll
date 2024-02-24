use std::{ffi::c_int, ptr};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ZArray<T>(*mut T);


#[derive(Debug)]
#[repr(C)]
pub struct ZArrayHeader {
    ref_count: c_int,
    cap: c_int,
    byte_len: c_int,
}


unsafe impl<T: Send> Send for ZArray<T> {}
unsafe impl<T: Sync> Sync for ZArray<T> {}

impl<T> ZArray<T> {
    pub fn empty() -> Self {
        unsafe { Self::from_ptr(ptr::null_mut()) }
    }

    pub unsafe fn from_ptr(ptr: *mut T) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut T {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }

    pub fn len(&self) -> usize {
        unsafe { self.header() }
            .map(|d| (d.byte_len as usize) / std::mem::size_of::<T>())
            .unwrap_or(0)
    }

    pub unsafe fn header_ptr(&self) -> *const ZArrayHeader {
        if self.0.is_null() {
            return ptr::null();
        }
        std::mem::transmute(self.0.byte_sub(0xC))
    }

    pub unsafe fn header(&self) -> Option<&ZArrayHeader> {
        self.header_ptr().as_ref()
    }

    pub fn data(&self) -> &[T] {
        let ln = self.len();
        if ln > 0 {
            unsafe { std::slice::from_raw_parts(self.0, ln) }
        } else {
            &[]
        }
    }
}