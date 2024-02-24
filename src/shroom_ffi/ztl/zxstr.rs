use std::ptr;

#[derive(Debug)]
#[repr(C, packed)]
pub struct ZXStringHeader {
    ref_count: i32,
    cap: i32,
    byte_len: i32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct ZXString<T>(pub *const T);
pub type ZXString8 = ZXString<u8>;
pub type ZXString16 = ZXString<u16>;

impl<T: PartialEq> PartialEq for ZXString<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data()
    }
}

impl<T: Eq> Eq for ZXString<T> {}

impl<T> ZXString<T> {
    pub fn str_len(&self) -> usize {
        let ln = self.len();
        if ln > 0 {
            //Remove null terminator
            ln
        } else {
            0
        }
    }

    pub fn empty() -> Self {
        Self::from_ptr(ptr::null())
    }

    pub fn from_ptr(ptr: *const T) -> Self {
        Self(ptr)
    }

    pub fn len(&self) -> usize {
        unsafe { self.header() }
            .map(|d| (d.byte_len as usize) / std::mem::size_of::<T>())
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }

    pub unsafe fn header_ptr(&self) -> *const ZXStringHeader {
        std::mem::transmute(self.0.byte_sub(0xC))
    }

    pub unsafe fn header(&self) -> Option<&ZXStringHeader> {
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

    pub fn data_str(&self) -> &[T] {
        let data = self.data();
        let str_len = self.str_len();

        &data[..str_len]
    }
}



impl ZXString<u8> {
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(self.data_str()).ok()
    }
}

impl ZXString<u16> {
    pub fn to_string_owned(&self) -> String {
        String::from_utf16_lossy(self.data_str())
    }
}