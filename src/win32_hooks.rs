use std::sync::atomic::{AtomicBool, Ordering};

use retour::GenericDetour;
use windows::{
    core::{s, w, PCSTR},
    Win32::{
        Foundation::{BOOL, HANDLE},
        Security::SECURITY_ATTRIBUTES,
        Storage::FileSystem::WIN32_FIND_DATAA,
    },
};

use crate::{
    static_win32_fn_hook,
    util::{hooks::HookModule, ref_time::RefTime},
};

pub struct Win32Hooks;

// This hook prevents that the client detects that there's a dinput8.dll in the clients directory
static_win32_fn_hook!(
    FIND_FIRST_FILE_A_HOOK,
    w!("kernel32.dll"),
    s!("FindFirstFileA"),
    find_first_file_detour,
    type FnFindFirstFileA = extern "system" fn(PCSTR, *mut WIN32_FIND_DATAA) -> HANDLE
);

extern "system" fn find_first_file_detour(
    mut file_name: PCSTR,
    find_file_data: *mut WIN32_FIND_DATAA,
) -> HANDLE {
    static SPOOFED_PROXY_DLL: AtomicBool = AtomicBool::new(false);

    if !file_name.is_null() && unsafe { file_name.as_bytes() } == b"*" {
        //Only spoof once at start
        if !SPOOFED_PROXY_DLL.fetch_or(true, Ordering::SeqCst) {
            log::info!("Spoofing FindFirstFileA for proxy dll");
            // Fake the file name with a dummy value
            file_name = s!("*.wz");
        }
    }
    FIND_FIRST_FILE_A_HOOK.call(file_name, find_file_data)
}

// Allows multiple client processes to launch
// by spoofing the mutex name

static_win32_fn_hook!(
    CREATE_MUTEX_A_HOOK,
    w!("kernel32.dll"),
    s!("CreateMutexA"),
    create_mutex_a_detour,
    type FnCreateMutexA = extern "system" fn(*const SECURITY_ATTRIBUTES, BOOL, PCSTR) -> HANDLE
);

extern "system" fn create_mutex_a_detour(
    lpmutexattributes: *const SECURITY_ATTRIBUTES,
    binitialowner: BOOL,
    name: PCSTR,
) -> HANDLE {
    if !name.is_null() {
        //TODO check if we can use a null for the name
        let name_s = unsafe { name.display() };
        let pid = std::process::id();

        let spoofed_mtx_name = format!("{name_s}_{pid}\0");
        log::info!("Spoofing Mutex to: {}", spoofed_mtx_name);

        return CREATE_MUTEX_A_HOOK.call(
            lpmutexattributes,
            binitialowner,
            PCSTR::from_raw(spoofed_mtx_name.as_ptr()),
        );
    }
    CREATE_MUTEX_A_HOOK.call(lpmutexattributes, binitialowner, name)
}

// Hook the time counters to let them start at 0
static_win32_fn_hook!(
    GET_TICK_COUNT_HOOK,
    w!("kernel32.dll"),
    s!("GetTickCount"),
    get_tick_count_hook,
    type FnGetTickCount = extern "system" fn() -> u32
);

extern "system" fn get_tick_count_hook() -> u32 {
    static REF_TICKS: RefTime = RefTime::new();
    let orig = GET_TICK_COUNT_HOOK.call();
    REF_TICKS.get_time(orig)
}

static_win32_fn_hook!(
    TIME_GET_TIME_HOOK,
    w!("Winmm.dll"),
    s!("timeGetTime"),
    time_get_time_hook,
    type FnTimeGetTime = extern "system" fn() -> u32
);

extern "system" fn time_get_time_hook() -> u32 {
    static REF_TICKS: RefTime = RefTime::new();
    let orig = TIME_GET_TIME_HOOK.call();
    REF_TICKS.get_time(orig)
}

impl HookModule for Win32Hooks {
    unsafe fn enable(&self) -> anyhow::Result<()> {
        FIND_FIRST_FILE_A_HOOK.enable()?;
        CREATE_MUTEX_A_HOOK.enable()?;
        GET_TICK_COUNT_HOOK.enable()?;
        TIME_GET_TIME_HOOK.enable()?;
        todo!()
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        FIND_FIRST_FILE_A_HOOK.disable()?;
        CREATE_MUTEX_A_HOOK.disable()?;
        GET_TICK_COUNT_HOOK.disable()?;
        TIME_GET_TIME_HOOK.disable()?;
        todo!()
    }
}
