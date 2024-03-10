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

use std::{ffi::c_void, fs::File};

use anyhow::Context;
use config::LogBackend;
use crossbeam::atomic::AtomicCell;
use log::LevelFilter;
use shroom_hooks::ShroomHooks;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode, WriteLogger};
use util::hooks::HookModule;
use win32_hooks::Win32Hooks;
use windows::{
    core::{s, IUnknown, GUID, HRESULT},
    Win32::{
        Foundation::{BOOL, HMODULE},
        System::{
            Console::AllocConsole, LibraryLoader::{GetProcAddress, LoadLibraryA}, SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH}
        },
    },
};

use crate::{config::CONFIG, login::LoginHooks, socket::PacketHooks, wz::WzHooks};

//pub mod net;
pub mod app;
pub mod config;
pub mod exceptions;
pub mod login;
#[cfg(feature = "overlay")]
pub mod overlay;
pub mod shroom_ffi;
pub mod shroom_hooks;
pub mod socket;
pub mod util;
pub mod win32_hooks;
pub mod wz;

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

fn setup_logs(backend: &LogBackend) -> anyhow::Result<()> {
    let filter = LevelFilter::Trace;
    let cfg = simplelog::Config::default();

    match backend {
        LogBackend::Stdout => {
            TermLogger::init(
                filter,
                cfg,
                TerminalMode::Mixed,
                ColorChoice::Auto,
            )?;
        }
        LogBackend::Console => {
            unsafe { AllocConsole() }.context("Alloc console")?;
            TermLogger::init(
                filter,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            )?;
        }
        LogBackend::File(file) => {
            let file = File::create(file)?;
            WriteLogger::init(filter, cfg, file)?;
        }
        LogBackend::Debug => {
            win_dbg_logger::init();
        }
    }

    Ok(())
}

fn run() {
    // Init the overlay If It's required
    #[cfg(feature = "overlay")]
    overlay::init_module(MODULE.load());

    log::info!("Running");
    let cfg = CONFIG.get().unwrap();

    unsafe { LoginHooks.enable_if(cfg.auto_login_data.is_some()) }.expect("Login hooks");
    unsafe { PacketHooks.enable_if(cfg.packet_tracing.is_some()) }.expect("Packet hooks");

    for extra_dll in &cfg.extra_dlls {
        if let Err(err) = unsafe { LoadLibraryA(extra_dll.as_pcstr()) } {
            log::error!("Failed to load extra dll: {:?} - {:?}", extra_dll, err);
        }
    }
}

fn load_cfg() -> anyhow::Result<config::Config> {
    let cfg_path = std::env::var("SHROOM_CONFIG").unwrap_or("config.toml".to_string());
    let file = std::fs::read_to_string(&cfg_path).context("Reading config file failed")?;
    Ok(toml::from_str(&file)?)
}

fn initialize(hmodule: HMODULE) -> anyhow::Result<()> {
    match load_cfg() {
        Ok(cfg) => {
            let cfg = CONFIG.get_or_init(|| cfg);

            // Setup the logger as console
            setup_logs(&cfg.log_backend)?;
        }
        Err(err) => {
            let cfg = config::Config::default();
            let cfg = CONFIG.get_or_init(|| cfg);

            // Setup the logger as console
            setup_logs(&cfg.log_backend)?;
            log::error!("Failed to load config: {:?} - Using default config", err);
        }
    }
    let cfg = CONFIG.get().unwrap();
    MODULE.store(hmodule);

    // Load the system dinput8.dll
    log::info!("Loading proxy dll");
    let dinput8_lib = util::load_sys_dll("dinput8.dll")?;
    let dinput8_create = unsafe { GetProcAddress(dinput8_lib, s!("DirectInput8Create")) }
        .context("Failed to get DirectInput8Create")?;
    DINPUT8_CREATE.store(unsafe { std::mem::transmute(dinput8_create) });
    log::info!("Loaded proxy dll");

    // Do the win32 patches
    unsafe {
        Win32Hooks.enable()?;
        ShroomHooks.enable()?;
        WzHooks.enable()?;
    }

    if cfg.handle_exceptions {
        exceptions::setup_exception_handler();
    }
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
