use std::ffi::c_void;

// Like ZRefCountedDummy
#[derive(Debug)]
#[repr(C)]
pub struct ZListEntryHeader<T> {
    vtable: *const c_void,
    next: *const ZListEntry<T>,
    prev: *const ZListEntry<T>,
    zref_vtable: *const c_void,
    value: T,
}

// Like ZRefCounted
#[derive(Debug)]
#[repr(C)]
pub struct ZListEntry<T>(pub *const T);

impl<T> ZListEntry<T> {
    fn header(&self) -> *const ZListEntryHeader<T> {
        unsafe { std::mem::transmute(self.0.byte_sub(0x10)) }
    }

    fn get_next(&self) -> *const ZListEntry<T> {
        unsafe { self.header().as_ref() }
            .map(|d| d.next)
            .unwrap_or(std::ptr::null())
    }

    fn get_prev(&self) -> *const ZListEntry<T> {
        unsafe { self.header().as_ref() }
            .map(|d| d.prev)
            .unwrap_or(std::ptr::null())
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ZList<T> {
    pub vtable: *const c_void,
    pub unknown: *const c_void, // Another vtable?
    pub count: usize,
    pub head: ZListEntry<T>,
    pub tail: ZListEntry<T>,
}

impl<T> ZList<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        std::iter::successors(Some(&self.head), move |cur| unsafe {
            cur.get_next().as_ref()
        })
        .map(|x| unsafe { x.0.as_ref().unwrap() })
    }

    pub fn rev_iter(&self) -> impl Iterator<Item = &T> + '_ {
        std::iter::successors(Some(&self.tail), move |cur| unsafe {
            cur.get_prev().as_ref()
        })
        .map(|x| unsafe { x.0.as_ref().unwrap() })
    }
}
