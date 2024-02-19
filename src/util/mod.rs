use std::ptr;

use windows::core::PCWSTR;
use windows::Win32::Foundation::HMODULE;

use region::Protection;

use self::hooks::HookModule;

pub mod hooks;
pub mod packet_schema;
pub mod profiler;
pub mod ref_time;
pub mod stack_walker;

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
pub fn load_sys_dll(library: &str) -> anyhow::Result<HMODULE> {
    use windows::core::HSTRING;
    use windows::Win32::System::LibraryLoader::LoadLibraryW;

    let sys_dir = get_sys_path()?.join(library);
    unsafe {
        LoadLibraryW(&HSTRING::from(sys_dir.as_os_str()))
            .map_err(|e| anyhow::anyhow!("Unable to load {}: {:?}", library, e))
    }
}

#[cfg(not(windows))]
pub fn load_sys_dll(library: &str) -> anyhow::Result<HMODULE> {
    anyhow::bail!("Not implemented");
}

#[cfg(windows)]
pub fn get_sys_path() -> anyhow::Result<std::path::PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::{Foundation::MAX_PATH, System::SystemInformation::GetSystemDirectoryW};
    let mut buf = [0; (MAX_PATH + 1) as usize];
    let n = unsafe { GetSystemDirectoryW(Some(&mut buf)) } as usize;
    if n == 0 {
        anyhow::bail!("Unable to get sys dir");
    }

    Ok(OsString::from_wide(&buf[..n]).into())
}

#[cfg(not(windows))]
pub fn get_sys_path() -> anyhow::Result<PathBuf> {
    anyhow::bail!("Not implemented");
}
