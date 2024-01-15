#![feature(
    link_llvm_intrinsics,
    naked_functions,
    strict_provenance,
    asm_const,
    lazy_cell
)]
// llvm_intinsics are required for the llvm.returnaddress intrinsic
// missing_safety_doc is mostly useless because everything is unsafe
// TODO should be disabled later
#![allow(internal_features, clippy::missing_safety_doc)]

use std::{ffi::c_void, fs::File, path::Path};

use anyhow::Context;
use crossbeam::atomic::AtomicCell;
use log::LevelFilter;
use shroom_hooks::ShroomHooks;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode, WriteLogger};
use util::{exceptions, hooks::HookModule};
use win32_hooks::Win32Hooks;
use windows::{
    core::{s, w, IUnknown, GUID, HRESULT},
    Win32::{
        Foundation::{BOOL, HMODULE},
        System::{
            Console::AllocConsole,
            LibraryLoader::GetProcAddress,
            SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
        },
    },
};

#[cfg(feature = "overlay")]
pub mod overlay;
pub mod util;

pub mod shroom_ffi;

pub mod shroom_hooks;
pub mod win32_hooks;

type FDirectInput8Create = unsafe extern "system" fn(
    hinst: HMODULE,
    dwversion: u32,
    riidltf: *const GUID,
    ppvout: *mut *mut c_void,
    punkouter: IUnknown,
) -> HRESULT;

static DINPUT8_CREATE: AtomicCell<Option<FDirectInput8Create>> = AtomicCell::new(None);
static MODULE: AtomicCell<HMODULE> = AtomicCell::new(HMODULE(0));

/// dinput8 exported function
#[no_mangle]
#[allow(unsupported_calling_conventions)]
unsafe extern "system" fn DirectInput8Create(
    hinst: HMODULE,
    dwversion: u32,
    riidltf: *const GUID,
    ppvout: *mut *mut c_void,
    punkouter: IUnknown,
) -> HRESULT {
    if let Some(ref dinput8_create) = DINPUT8_CREATE.load() {
        return dinput8_create(hinst, dwversion, riidltf, ppvout, punkouter);
    }

    log::error!("DirectInput8Create called before initialization");
    panic!("DirectInput8Create called before initialization");
}

fn setup_logs<T: AsRef<Path>>(file: Option<T>) -> anyhow::Result<()> {
    let filter = LevelFilter::Trace;
    let cfg = simplelog::Config::default();

    if let Some(file) = file {
        let file = File::create(file.as_ref())?;
        WriteLogger::init(filter, cfg, file)?;
    } else {
        unsafe { AllocConsole() }.context("Alloc console")?;
        TermLogger::init(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )?;
    }

    Ok(())
}

fn run() {
    // Init the overlay If It's required
    #[cfg(feature = "overlay")]
    overlay::init_module(MODULE.load());

    if let Err(err) = unsafe { ShroomHooks.enable() } {
        log::error!("Failed to enable shroom hooks: {:?}", err);
    }
}

fn initialize(hmodule: HMODULE) -> anyhow::Result<()> {
    MODULE.store(hmodule);

    // Setup the logger as console
    setup_logs::<&str>(None)?;

    // Load the system dinput8.dll
    let dinput8_lib = util::load_sys_dll(w!("dinput8.dll"))?;
    let dinput8_create = unsafe { GetProcAddress(dinput8_lib, s!("DirectInput8Create")) }
        .context("Failed to get DirectInput8Create")?;
    DINPUT8_CREATE.store(unsafe { std::mem::transmute(dinput8_create) });

    // Do the win32 patches
    unsafe {
        Win32Hooks.enable()?;
    }

    exceptions::setup_exception_handler();

    // Launch run in a new thread, so we don't block the main thread
    std::thread::spawn(run);

    Ok(())
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(hmodule: HMODULE, call_reason: u32, reserved: *mut c_void) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            if let Err(err) = initialize(hmodule) {
                log::error!("Failed to initialize proxy dll: {:?}", err);
                return BOOL::from(false);
            }
        }
        DLL_PROCESS_DETACH => {
            log::info!("Detaching proxy dll");
        }
        _ => (),
    }

    BOOL::from(true)
}
