use std::{
    ffi::{c_int, c_uint, c_void, CStr},
    fs::OpenOptions,
    io::Write,
};

use chrono::Local;
use windows::Win32::{
    Foundation::NTSTATUS,
    System::{
        Diagnostics::Debug::{AddVectoredExceptionHandler, EXCEPTION_POINTERS, EXCEPTION_RECORD},
        Kernel::ExceptionContinueSearch,
    },
};

use crate::{
    config::CONFIG,
    shroom_ffi::{
        error_codes::ClientErrorCode,
        ztl::{ZException, ZEXCEPTION_MAGIC},
    },
    util::stack_walker::StackWalker,
};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CxxPMD {
    pub mdisp: c_int,
    pub pdisp: c_int,
    pub vdisp: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CxxTypeDescriptor {
    pub vf_table: *const c_void,
    pub spare: *const c_void,
    pub name: [u8; 0],
}

impl CxxTypeDescriptor {
    fn name(&self) -> Option<&CStr> {
        const MAX_TYPE_NAME: usize = 64;
        unsafe {
            let data = std::slice::from_raw_parts(self.name.as_ptr(), MAX_TYPE_NAME);
            CStr::from_bytes_until_nul(data).ok()
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct CxxCatchableType {
    pub properties: c_uint,
    pub ty: *mut CxxTypeDescriptor,
    pub this_displacement: CxxPMD,
    pub size_or_offset: c_uint,
    pub copy_fn: *mut c_void,
}

impl CxxCatchableType {
    pub fn name(&self) -> Option<&CStr> {
        unsafe { self.ty.as_ref().and_then(|ty| ty.name()) }
    }
}

#[repr(C)]
struct CxxCatchableTypeArray {
    pub count: c_int,
    pub types: *mut CxxCatchableType,
}

impl CxxCatchableTypeArray {
    pub unsafe fn types(&self) -> &[CxxCatchableType] {
        std::slice::from_raw_parts(self.types as *const CxxCatchableType, self.count as usize)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct CxxThrowInfo {
    pub attributes: c_uint,
    pub unwind_fn: *mut c_void,
    pub forward_compat_fn: *mut c_void,
    pub catchable_type_array: *mut CxxCatchableTypeArray,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct CxxThrowException {
    pub magic: u32,
    pub obj: *mut c_void,
    pub throw_info: *mut CxxThrowInfo,
}

static_assertions::assert_eq_size!(CxxThrowException, [usize; 3]);
impl From<[usize; 3]> for CxxThrowException {
    fn from(value: [usize; 3]) -> Self {
        unsafe { std::mem::transmute_copy(&value) }
    }
}

impl TryFrom<&EXCEPTION_RECORD> for CxxThrowException {
    type Error = ();

    fn try_from(value: &EXCEPTION_RECORD) -> Result<Self, Self::Error> {
        if value.ExceptionCode == MSVCXX_EXCEPTION_CODE && value.NumberParameters == 3 {
            Ok(<[usize; 3]>::try_from(&value.ExceptionInformation[..3])
                .unwrap()
                .into())
        } else {
            Err(())
        }
    }
}

const MSVCXX_EXCEPTION_CODE: NTSTATUS = NTSTATUS(0xe06d7363u32 as i32);

impl CxxThrowException {
    pub fn types(&self) -> &[CxxCatchableType] {
        unsafe {
            self.throw_info
                .as_ref()
                .and_then(|info| info.catchable_type_array.as_ref())
                .map(|arr| arr.types())
                .unwrap_or(&[])
        }
    }

    pub fn as_zexception(&self) -> Option<&ZException> {
        if self.magic == ZEXCEPTION_MAGIC {
            unsafe { std::mem::transmute::<_, *const ZException>(self.obj).as_ref() }
        } else {
            None
        }
    }
}

fn write_trace(
    file: &str,
    cxx_ex: Option<CxxThrowException>,
    walker: Option<StackWalker>,
) -> std::io::Result<()> {
    let mut f = OpenOptions::new().create(true).append(true).open(file)?;

    writeln!(f, "{:-<80}", "")?;
    writeln!(f, "Exception at {}", Local::now())?;

    if let Some(cxx_ex) = cxx_ex {
        if let Some(zex) = cxx_ex.as_zexception() {
            let hres = zex.0;
            write!(f, "ZException: {hres:?}")?;

            if let Ok(ec) = ClientErrorCode::try_from(hres.0 as u32) {
                write!(f, "\tClientErrorCode: {:?}", ec)?;
            }

            let msg = hres.message();
            if !msg.is_empty() {
                write!(f, "\tMessage: {}", msg)?;
            }

            writeln!(f)?;
        }

        for (i, ty) in cxx_ex.types().iter().enumerate() {
            if let Some(name) = ty.name() {
                writeln!(f, "Type({i}): {name:?}")?;
            }
        }
    }

    struct SymName<'a>(Option<&'a CStr>);
    impl<'a> std::fmt::Display for SymName<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self.0 {
                Some(name) => write!(f, "<{}>", name.to_bytes().escape_ascii()),
                None => write!(f, "<unknown>"),
            }
        }
    }

    if let Some(mut walker) = walker {
        let mut i = 0;
        while let Ok(Some(frame)) = walker.get_next_frame() {
            if i == 0 {
                writeln!(f, "Trace:")?;
            }

            if i > 8 {
                writeln!(f, "\t...")?;
                break;
            }

            writeln!(
                f,
                "\t{:p}:\t{}",
                frame.ret as *const c_void,
                SymName(frame.sym),
            )?;

            i += 1;
        }
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct ExceptionHandler {
    init_sym: bool
}

impl ExceptionHandler {
    pub const fn new() -> Self {
        ExceptionHandler {
            init_sym: false
        }
    }

    pub unsafe fn handle_ex(&mut self, info: &EXCEPTION_POINTERS, record: &EXCEPTION_RECORD) {
        let walker = info.ContextRecord.as_ref().map(|ctx| {
            let walker = StackWalker::from_ctx(*ctx);
            if !self.init_sym {
                self.init_sym = true;
                if let Err(err) = load_init_sym(&walker) {
                    log::error!("Failed to init sym: {:?}", err);
                };
            }
            walker
        });
    
        let _ = write_trace(
            "exception_log.txt",
            CxxThrowException::try_from(record).ok(),
            walker,
        );
    }
}

static EXCEPTION_HANDLER: std::sync::Mutex<ExceptionHandler> = std::sync::Mutex::new(ExceptionHandler::new());


fn load_init_sym(sw: &StackWalker) -> windows::core::Result<()> {
    sw.sym_init()?;
    let cfg = CONFIG.get().unwrap();
    if let Some(ref pdb_file) = cfg.pdb_file {
        sw.sym_load_main_pdb(pdb_file.as_pcstr())?;
    }
    Ok(())
}

unsafe extern "system" fn exception_handler(exception_info: *mut EXCEPTION_POINTERS) -> i32 {
    let Some(info) = exception_info.as_ref() else {
        return ExceptionContinueSearch.0;
    };

    let Some(record) = info.ExceptionRecord.as_ref() else {
        return ExceptionContinueSearch.0;
    };

    // Weird cpu error we skip
    if record.ExceptionCode == NTSTATUS(1080890248) {
        return ExceptionContinueSearch.0;
    }

    log::error!("Exception at: {:p} - code: {:?}", record.ExceptionAddress, record.ExceptionCode);

    if let Ok(mut handler) = EXCEPTION_HANDLER.try_lock() {
        handler.handle_ex(info, record);
    }

    ExceptionContinueSearch.0
}

pub fn setup_exception_handler() {
    unsafe {
        AddVectoredExceptionHandler(1, Some(exception_handler));
    }
}
