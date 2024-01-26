use std::ffi::c_void;

use super::zxstr::{ZXString16, ZXString8};

/*
TODOs
    - combine: (((a << 5) + b) << 5) + c + 0x421
    - raw string is like ZXString but without the null terminator

*/

pub trait ZMapKey: Eq {
    fn zhash(&self) -> u32;
}

impl ZMapKey for i16 {
    fn zhash(&self) -> u32 {
        (self.rotate_right(5)) as u32
    }
}

impl ZMapKey for u16 {
    fn zhash(&self) -> u32 {
        (self.rotate_right(5)) as u32
    }
}

impl ZMapKey for i32 {
    fn zhash(&self) -> u32 {
        (self.rotate_right(5)) as u32
    }
}

impl ZMapKey for u32 {
    fn zhash(&self) -> u32 {
        self.rotate_right(5)
    }
}

impl ZMapKey for i64 {
    fn zhash(&self) -> u32 {
        (*self as i32).zhash()
    }
}

impl ZMapKey for u64 {
    fn zhash(&self) -> u32 {
        (*self as u32).zhash()
    }
}

impl ZMapKey for ZXString8 {
    fn zhash(&self) -> u32 {
        self.data()
            .iter()
            .fold(0u32, |acc, &c| acc.wrapping_add(1 + (c << 5) as u32))
    }
}

impl ZMapKey for ZXString16 {
    fn zhash(&self) -> u32 {
        self.data()
            .iter()
            .fold(0u32, |acc, &c| acc.wrapping_add(1 + (c << 5) as u32))
    }
}

// TODO: check If packed is required or not
#[derive(Debug)]
#[repr(C)]
pub struct ZMapPair<K, V> {
    pub vtable: *const c_void,
    pub next: *const ZMapPair<K, V>,
    pub key: K,
    pub value: V,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct ZMap<K: ZMapKey, V> {
    pub vtable: *const c_void,
    pub apair_table: *const *const ZMapPair<K, V>,
    pub table_size: u32,
    pub count: u32,
    pub auto_grow_every_128: u32,
    pub auto_grow_limit: u32,
}

impl<K: ZMapKey, V> ZMap<K, V> {
    fn tables(&self) -> &[*const ZMapPair<K, V>] {
        unsafe { std::slice::from_raw_parts(self.apair_table, self.table_size as usize) }
    }

    fn traverse_bucket(&self, ix: usize) -> impl Iterator<Item = &ZMapPair<K, V>> {
        let table = self.tables()[ix];
        std::iter::successors(unsafe { table.as_ref() }, move |cur| unsafe {
            cur.next.as_ref()
        })
    }

    pub fn get_at(&self, key: &K) -> Option<&V> {
        let hash = key.zhash();
        self.traverse_bucket(hash as usize % self.table_size as usize)
            .find(|pair| &pair.key == key)
            .map(|pair| &pair.value)
    }
}
