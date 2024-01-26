use std::ffi::CStr;

use windows::core::PCSTR;
use windows::Win32::Foundation::{HMODULE, MAX_PATH};
use windows::Win32::System::Diagnostics::Debug::{
    SymCleanup, SymInitialize, SymLoadModuleEx, SYM_LOAD_FLAGS,
};
use windows::Win32::System::Environment::GetCurrentDirectoryA;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::ProcessStatus::{GetModuleInformation, MODULEINFO};
use windows::{
    core::Error,
    Win32::{
        Foundation::HANDLE,
        System::{
            Diagnostics::Debug::{
                GetThreadContext, RtlCaptureContext, SymGetSymFromAddr64, CONTEXT, CONTEXT_ALL_X86,
                CONTEXT_FULL_X86, IMAGEHLP_SYMBOL64,
            },
            Threading::{
                GetCurrentProcess, GetCurrentThread, GetCurrentThreadId, GetThreadId, ResumeThread,
                SuspendThread,
            },
        },
    },
};

use self::arch::AddrTy;

const MAX_RECURSIONS: usize = 3;

#[cfg(target_arch = "x86_64")]
mod arch {
    use std::ffi::c_void;

    use windows::Win32::{
        Foundation::HANDLE,
        System::{
            Diagnostics::Debug::{
                AddrModeFlat, StackWalkEx, SymFunctionTableAccess64, SymGetModuleBase64, CONTEXT,
                STACKFRAME_EX,
            },
            SystemInformation::IMAGE_FILE_MACHINE_AMD64,
        },
    };

    pub type AddrTy = u64;
    pub type StackFrameTy = STACKFRAME_EX;

    pub unsafe extern "system" fn get_module_base(hproc: HANDLE, addr: AddrTy) -> AddrTy {
        SymGetModuleBase64(hproc, addr)
    }

    pub unsafe extern "system" fn ftable_access(hproc: HANDLE, addr: AddrTy) -> *mut c_void {
        SymFunctionTableAccess64(hproc, addr)
    }

    pub unsafe fn get_first_frame(ctx: &mut CONTEXT) -> STACKFRAME_EX {
        let mut stack_frame = STACKFRAME_EX::default();
        stack_frame.AddrPC.Offset = ctx.Rip as AddrTy;
        stack_frame.AddrPC.Mode = AddrModeFlat;

        stack_frame.AddrFrame.Offset = ctx.Rbp as AddrTy;
        stack_frame.AddrFrame.Mode = AddrModeFlat;

        stack_frame.AddrStack.Offset = ctx.Rsp as AddrTy;
        stack_frame.AddrStack.Mode = AddrModeFlat;

        stack_frame
    }

    pub unsafe fn stack_walk(
        proc: HANDLE,
        thread: HANDLE,
        frame: &mut STACKFRAME_EX,
        ctx: *mut CONTEXT,
    ) -> bool {
        StackWalkEx(
            IMAGE_FILE_MACHINE_AMD64.0 as u32,
            proc,
            thread,
            frame,
            ctx as *mut c_void,
            None,
            Some(ftable_access),
            Some(get_module_base),
            None,
            0,
        )
        .as_bool()
    }
}

#[cfg(target_arch = "x86")]
mod arch {
    use std::ffi::c_void;

    use windows::Win32::{
        Foundation::HANDLE,
        System::{
            Diagnostics::Debug::{
                AddrModeFlat, StackWalk, SymFunctionTableAccess, SymGetModuleBase, CONTEXT,
                STACKFRAME,
            },
            SystemInformation::IMAGE_FILE_MACHINE_I386,
        },
    };

    pub type StackFrameTy = STACKFRAME;
    pub type AddrTy = u32;

    pub unsafe extern "system" fn get_module_base(hproc: HANDLE, addr: AddrTy) -> AddrTy {
        SymGetModuleBase(hproc, addr)
    }

    pub unsafe extern "system" fn ftable_access(hproc: HANDLE, addr: AddrTy) -> *mut c_void {
        SymFunctionTableAccess(hproc, addr)
    }

    pub fn get_first_frame(ctx: &mut CONTEXT) -> STACKFRAME {
        let mut stack_frame = STACKFRAME::default();
        stack_frame.AddrPC.Offset = ctx.Eip as AddrTy;
        stack_frame.AddrPC.Mode = AddrModeFlat;

        stack_frame.AddrFrame.Offset = ctx.Ebp as AddrTy;
        stack_frame.AddrFrame.Mode = AddrModeFlat;

        stack_frame.AddrStack.Offset = ctx.Esp as AddrTy;
        stack_frame.AddrStack.Mode = AddrModeFlat;

        stack_frame
    }

    pub unsafe fn stack_walk(
        proc: HANDLE,
        thread: HANDLE,
        frame: &mut STACKFRAME,
        ctx: &mut CONTEXT,
    ) -> bool {
        StackWalk(
            IMAGE_FILE_MACHINE_I386.0 as u32,
            proc,
            thread,
            frame,
            ctx as *mut CONTEXT as *mut c_void,
            None,
            Some(ftable_access),
            Some(get_module_base),
            None,
        )
        .as_bool()
    }
}

#[allow(unused)]
unsafe fn capture_thread_context(thread: HANDLE) -> windows::core::Result<CONTEXT> {
    if GetThreadId(thread) == GetCurrentThreadId() {
        let mut ctx = CONTEXT {
            ContextFlags: CONTEXT_ALL_X86,
            ..Default::default()
        };
        RtlCaptureContext(&mut ctx);
        return Ok(ctx);
    }

    if SuspendThread(thread) == u32::MAX {
        return Err(Error::from_win32());
    }
    let mut ctx = CONTEXT {
        ContextFlags: CONTEXT_FULL_X86,
        ..Default::default()
    };
    let res = GetThreadContext(thread, &mut ctx);
    if ResumeThread(thread) == u32::MAX {
        return Err(Error::from_win32());
    }

    res.map(|_| ctx)
}

#[derive(Debug)]
pub struct StackFrame<'a> {
    pub pc: AddrTy,
    pub ret: AddrTy,
    pub frame: AddrTy,
    pub sym: Option<&'a CStr>,
}

const MAX_SYM_NAME: usize = 512;

#[repr(C)]
pub struct ImgHlpSym64 {
    base: IMAGEHLP_SYMBOL64,
    name_buf: [u8; MAX_SYM_NAME],
}

impl ImgHlpSym64 {
    pub fn name(&self) -> Option<&CStr> {
        let buf = unsafe { std::slice::from_raw_parts(self.base.Name.as_ptr(), MAX_SYM_NAME) };
        CStr::from_bytes_until_nul(buf).ok()
    }
}

impl Default for ImgHlpSym64 {
    fn default() -> Self {
        Self {
            base: IMAGEHLP_SYMBOL64 {
                SizeOfStruct: std::mem::size_of::<ImgHlpSym64>() as u32,
                MaxNameLength: MAX_SYM_NAME as u32,
                ..Default::default()
            },
            name_buf: [0; MAX_SYM_NAME],
        }
    }
}

pub struct StackWalker {
    proc: HANDLE,
    thread: HANDLE,
    recursions: usize,
    last_frame: arch::StackFrameTy,
    ctx: CONTEXT,
    sym: ImgHlpSym64,
    has_sym: bool,
}

impl StackWalker {
    pub fn new(proc: HANDLE, thread: HANDLE, mut ctx: CONTEXT) -> Self {
        let last_frame = arch::get_first_frame(&mut ctx);
        Self {
            proc,
            thread,
            last_frame,
            ctx,
            recursions: 0,
            sym: ImgHlpSym64::default(),
            has_sym: false,
        }
    }

    pub fn from_ctx(ctx: CONTEXT) -> Self {
        unsafe {
            let proc = GetCurrentProcess();
            let thread = GetCurrentThread();
            Self::new(proc, thread, ctx)
        }
    }

    pub fn sym_cleanup(&self) -> windows::core::Result<()> {
        unsafe { SymCleanup(self.proc) }
    }

    pub fn sym_init(&self) -> windows::core::Result<()> {
        let mut cur = [0; MAX_PATH as usize];
        unsafe {
            GetCurrentDirectoryA(Some(&mut cur));
            SymInitialize(self.proc, PCSTR(cur.as_ptr()), true)?;
        }

        Ok(())
    }

    pub fn sym_load_main_pdb(&self, path: PCSTR) -> windows::core::Result<()> {
        let module = unsafe { GetModuleHandleW(None) }?;
        self.sym_load_pdb(module, path)
    }

    pub fn sym_load_pdb(&self, module: HMODULE, path: PCSTR) -> windows::core::Result<()> {
        let mut module_info = MODULEINFO::default();
        unsafe {
            GetModuleInformation(
                self.proc,
                module,
                &mut module_info,
                std::mem::size_of::<MODULEINFO>() as u32,
            )?
        };

        let res = unsafe {
            SymLoadModuleEx(
                self.proc,
                None,
                path,
                PCSTR::null(),
                module.0 as u64,
                module_info.SizeOfImage,
                None,
                SYM_LOAD_FLAGS::default(),
            )
        };

        match res {
            0 => Err(Error::from_win32()),
            _ => Ok(()),
        }
    }

    fn load_symbol(&mut self) -> windows::core::Result<()> {
        self.has_sym = false;
        let pc = self.last_frame.AddrReturn.Offset as u64;

        // Get symbol
        let mut sym_offset = 0;
        self.sym = ImgHlpSym64::default();
        self.sym.base.Address = pc;
        unsafe { SymGetSymFromAddr64(self.proc, pc, Some(&mut sym_offset), &mut self.sym.base) }?;
        self.has_sym = true;
        Ok(())
    }

    fn next_frame(&mut self) -> windows::core::Result<bool> {
        let res = unsafe {
            arch::stack_walk(self.proc, self.thread, &mut self.last_frame, &mut self.ctx)
        };

        if !res {
            return Ok(false);
        }

        if self.last_frame.AddrPC.Offset == self.last_frame.AddrReturn.Offset {
            if self.recursions > MAX_RECURSIONS {
                // TODO: maybe use an actual error later
                return Ok(false);
            }
            self.recursions += 1;
        } else {
            self.recursions = 0;
        }
        let _ = self.load_symbol();

        Ok(true)
    }

    pub fn get_sym(&self) -> Option<&ImgHlpSym64> {
        self.has_sym.then_some(&self.sym)
    }

    pub fn get_frame(&self) -> StackFrame {
        let pc = self.last_frame.AddrPC.Offset;
        let ret = self.last_frame.AddrReturn.Offset;
        let frame = self.last_frame.AddrFrame.Offset;
        let sym = self.get_sym().and_then(|sym| sym.name());

        StackFrame {
            pc,
            ret,
            frame,
            sym,
        }
    }

    pub fn get_next_frame(&mut self) -> windows::core::Result<Option<StackFrame>> {
        Ok(self.next_frame()?.then(|| self.get_frame()))
    }
}
