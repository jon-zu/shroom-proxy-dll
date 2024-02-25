use std::{
    ffi::{c_char, c_int, c_uchar, c_void},
    time::Instant,
};

use widestring::U16CString;
use windows::core::{HRESULT, PCWSTR};

use crate::{
    config::CONFIG,
    hook_list, lazy_hook,
    shroom_ffi::{
        self, bstr_assign, ztl::zxstr::ZXString8, CiobufferManipulatorDe, CiobufferManipulatorEn, IWzFileSystem, IWzPackage, IWzSeekableArchive, ZtlBstrT
    },
    static_lazy_hook,
    util::hooks::{HookModule, LazyHook},
};

static CMSGBOX_HOOK: LazyHook<shroom_ffi::CmsgboxInit> =
    lazy_hook!(shroom_ffi::cmsgbox_init, cmsgbox_init_hook);

unsafe extern "thiscall" fn cmsgbox_init_hook(
    this: *mut c_void,
    msg: ZXString8,
    link: ZXString8,
    desc: ZXString8,
) {
    log::info!(
        "msg box: {:?}, {:?} {:?}",
        msg.as_str(),
        link.as_str(),
        desc.as_str()
    );
    CMSGBOX_HOOK.call(this, msg, link, desc)
}

static SKIP_LOGO_HOOK: LazyHook<shroom_ffi::ClogoInit> =
    lazy_hook!(shroom_ffi::clogo_init, clogo_init_hook);
unsafe extern "thiscall" fn clogo_init_hook(this: *mut shroom_ffi::CLogo, _param: *const c_void) {
    shroom_ffi::clogo_end()(this);
}

static CWVS_APP_INITIALIZE_GAME_DATA_HOOK: LazyHook<shroom_ffi::CwvsAppInitializeGameData> = lazy_hook!(
    shroom_ffi::cwvs_app_initialize_game_data,
    cwvs_app_initialize_game_data_hook
);

unsafe extern "thiscall" fn cwvs_app_initialize_game_data_hook(this: *mut shroom_ffi::CWvsApp) {
    let start = Instant::now();
    CWVS_APP_INITIALIZE_GAME_DATA_HOOK.call(this);
    log::info!("cwvs_app_initialize_game_data took {:?}", start.elapsed());
}

static CIOBUFFER_MANIPULATOR_EN_HOOK: LazyHook<CiobufferManipulatorEn> = lazy_hook!(
    shroom_ffi::ciobuffer_manipulator_en,
    ciobuffer_manipulator_en_hook
);

unsafe extern "stdcall" fn ciobuffer_manipulator_en_hook(_buf: *mut c_char, _ln: c_int) -> c_uchar {
    1
}

static CIOBUFFER_MANIPULATOR_DE_HOOK: LazyHook<CiobufferManipulatorDe> = lazy_hook!(
    shroom_ffi::ciobuffer_manipulator_de,
    ciobuffer_manipulator_de_hook
);

unsafe extern "stdcall" fn ciobuffer_manipulator_de_hook(_buf: *mut c_char, _ln: c_int) -> c_uchar {
    1
}

static_lazy_hook!(
    WZ_PACKAGE_HOOK,
    shroom_ffi::IwzPackageInitRef,
    wz_package_hook
);

unsafe extern "thiscall" fn wz_package_hook(
    this: *mut IWzPackage,
    mut key: ZtlBstrT,
    base_uol: ZtlBstrT,
    archive: *mut IWzSeekableArchive,
) -> HRESULT {
    let cfg = CONFIG.get().unwrap();
    if let Some(version) = cfg.wz.as_ref().map(|wz| &wz.version) {
        bstr_assign()(&mut key as *mut _, version.as_pcwstr());
    }
    let key_ = key.as_wstr().map(|s| s.to_string());
    let base_uol_ = base_uol.as_wstr().map(|s| s.to_string());
    log::info!("wz_package_hook: {:?} {:?}", key_, base_uol_);
    WZ_PACKAGE_HOOK.call(this, key, base_uol, archive)
}

static_lazy_hook!(
    WZ_FS_HOOK,
    shroom_ffi::IwzFilesystemInitRef,
    wz_fs_hook
);

unsafe extern "thiscall" fn wz_fs_hook(
    this: *mut IWzFileSystem,
    mut path: ZtlBstrT,
) -> HRESULT {
    let cfg = CONFIG.get().unwrap();
    log::info!("wz fs init: {:?}", path.as_wstr().map(|s| s.to_string()));
    if let Some(new_path) = cfg.wz.as_ref().and_then(|wz| wz.path.as_ref()) {
        let current_dir = std::env::current_dir().unwrap();
        let new_path = current_dir.join(new_path);
        
        let str = U16CString::from_os_str_truncate(new_path);
        log::info!("wz fs new path: {:?}", str.to_string_lossy());
        bstr_assign()(&mut path as *mut _, PCWSTR(str.as_ptr()));
    }
    WZ_FS_HOOK.call(this, path)
}

hook_list!(
    ShandaHooks,
    CIOBUFFER_MANIPULATOR_EN_HOOK,
    CIOBUFFER_MANIPULATOR_DE_HOOK,
);

hook_list!(
    WzHooks,
    WZ_PACKAGE_HOOK,
    WZ_FS_HOOK,
);

pub struct ShroomHooks;

impl HookModule for ShroomHooks {
    unsafe fn enable(&self) -> anyhow::Result<()> {
        let cfg = CONFIG.get().unwrap();
        SKIP_LOGO_HOOK.enable_if(cfg.skip_logo)?;
        CMSGBOX_HOOK.enable_if(cfg.log_msgbox)?;
        CWVS_APP_INITIALIZE_GAME_DATA_HOOK.enable()?;
        ShandaHooks.enable_if(cfg.disable_shanda)?;
        WzHooks.enable_if(cfg.wz.is_some())?;

        Ok(())
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        let cfg = CONFIG.get().unwrap();
        SKIP_LOGO_HOOK.disable_if(cfg.skip_logo)?;
        CMSGBOX_HOOK.disable_if(cfg.log_msgbox)?;
        CWVS_APP_INITIALIZE_GAME_DATA_HOOK.disable()?;
        ShandaHooks.disable_if(cfg.disable_shanda)?;
        WzHooks.disable_if(cfg.wz.is_some())?;

        Ok(())
    }
}
