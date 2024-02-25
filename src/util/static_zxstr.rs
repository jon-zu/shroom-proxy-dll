use std::sync::atomic::{AtomicI32, Ordering};

use crate::shroom_ffi::ztl::zxstr::ZXString8;

#[repr(C)]
pub struct StaticZXStringData<const N: usize> {
    ref_count: AtomicI32,
    cap: i32,
    byte_len: i32,
    data: [u8; N],
    fake_zero: u8
}

impl<const N: usize> StaticZXStringData<N> {
    pub const fn new(s: &[u8; N]) -> Self {
        Self {
            ref_count: AtomicI32::new(1),
            cap: N as i32,
            byte_len: N as i32,
            data: *s,
            fake_zero: 0
        }
    }

    pub fn get(&'static self) -> ZXString8 {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
        ZXString8::from_ptr(self.data.as_ptr())
    }
}


macro_rules! static_ref_string {
    ($name:ident, $len:expr, $lit:literal) => {
        pub static $name: StaticZXStringData<$len> = StaticZXStringData::new($lit);
    }
}


static_ref_string!(S_1, 11, b"Hello World");