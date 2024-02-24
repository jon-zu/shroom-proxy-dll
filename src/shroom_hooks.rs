use std::{
    ffi::{c_char, c_int, c_uchar, c_void}, time::Instant
};

use crate::{
    config::CONFIG,
    hook_list, lazy_hook,
    shroom_ffi::{
        self, ztl::zxstr::ZXString8, CiobufferManipulatorDe, CiobufferManipulatorEn,
        
    },
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


hook_list!(
    ShandaHooks,
    CIOBUFFER_MANIPULATOR_EN_HOOK,
    CIOBUFFER_MANIPULATOR_DE_HOOK,
);

pub struct ShroomHooks;

impl HookModule for ShroomHooks {
    unsafe fn enable(&self) -> anyhow::Result<()> {
        let cfg = CONFIG.get().unwrap();
        SKIP_LOGO_HOOK.enable_if(cfg.skip_logo)?;
        CMSGBOX_HOOK.enable_if(cfg.log_msgbox)?;
        CWVS_APP_INITIALIZE_GAME_DATA_HOOK.enable()?;
        ShandaHooks.enable_if(cfg.disable_shanda)?;

        //GET_STRING_HOOK.enable().unwrap();

        Ok(())
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        let cfg = CONFIG.get().unwrap();
        SKIP_LOGO_HOOK.disable_if(cfg.skip_logo)?;
        CMSGBOX_HOOK.disable_if(cfg.log_msgbox)?;
        CWVS_APP_INITIALIZE_GAME_DATA_HOOK.disable()?;
        ShandaHooks.enable_if(cfg.disable_shanda)?;
        Ok(())
    }
}
