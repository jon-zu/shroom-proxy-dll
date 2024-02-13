use std::ptr;

use windows::core::PCWSTR;
use windows::Win32::Foundation::HMODULE;

use region::Protection;

use self::hooks::HookModule;

pub mod hooks;
pub mod ref_time;
pub mod stack_walker;
pub mod profiler;
pub mod packet_schema;

extern "C" {
    #[link_name = "llvm.returnaddress"]
    pub fn return_address(level: i32) -> *const u8;
}

/// Helper macro to get the return address of the current function.
#[macro_export]
macro_rules! ret_addr {
    () => {
        unsafe { $crate::util::return_address(0) as usize }
    };
    ($level:expr) => {
        unsafe { $crate::util::return_address($level) as usize }
    };
}

/// Memset function, with overwriting memory protection
pub unsafe fn ms_memset(mut addr: *mut u8, b: u8, cnt: usize) -> region::Result<()> {
    let _handle = region::protect_with_handle(addr, cnt, Protection::READ_WRITE_EXECUTE)?;

    for _ in 0..cnt {
        addr.write_volatile(b);
        addr = addr.offset(1);
    }

    Ok(())
}

/// Memcpy function, with overwriting memory protection
pub unsafe fn ms_memcpy(addr: *mut u8, src: *const u8, cnt: usize) -> region::Result<()> {
    let _handle = region::protect_with_handle(addr, cnt, Protection::READ_WRITE_EXECUTE)?;

    ptr::copy(src, addr, cnt);
    Ok(())
}

/// Writes n NOPs to addr
pub unsafe fn nop(addr: *mut u8, n: usize) -> region::Result<()> {
    ms_memset(addr, 0x90, n)
}

/// Simple mem patch, which saves the bytes before patching it
pub struct MemPatch<const N: usize> {
    addr: *const u8,
    patch: [u8; N],
    orig: [u8; N],
}

impl<const N: usize> MemPatch<N> {
    pub unsafe fn new(addr: *const u8, patch: [u8; N]) -> Self {
        let mut orig = [0; N];
        unsafe { addr.copy_to_nonoverlapping(orig.as_mut_ptr(), N) };

        Self { addr, patch, orig }
    }
}

impl<const N: usize> HookModule for MemPatch<N> {
    unsafe fn enable(&self) -> anyhow::Result<()> {
        ms_memcpy(self.addr as *mut u8, self.patch.as_ptr(), N)?;
        Ok(())
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        ms_memcpy(self.addr as *mut u8, self.orig.as_ptr(), N)?;
        Ok(())
    }
}

#[cfg(windows)]
pub fn load_sys_dll(library: PCWSTR) -> anyhow::Result<HMODULE> {
    use anyhow::Context;
    use windows::Win32::{
        Foundation::HANDLE,
        System::LibraryLoader::{LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32},
    };

    unsafe {
        LoadLibraryExW(library, HANDLE::default(), LOAD_LIBRARY_SEARCH_SYSTEM32)
            .context("Load sys library")
    }
}

#[cfg(not(windows))]
pub fn load_sys_dll(_library: PCWSTR) -> anyhow::Result<HMODULE> {
    unimplemented!("Not implemented");
}
