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
    pub fn get(&self) -> ZXString8 {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
        ZXString8::from_ptr(self.data.as_ptr())
    }
}


macro_rules! static_ref_string {
    ($name:ident, $len:expr, $lit:literal) => {
        pub static $name: StaticZXStringData<$len> = StaticZXStringData {
            ref_count: AtomicI32::new(1),
            cap: $len,
            byte_len: $len,
            data: *$lit,
            fake_zero: 0
        };
    }
}


static_ref_string!(S_1, 11, b"Hello World");