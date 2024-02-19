use std::{
    ffi::{c_char, c_int, c_uchar, c_void},
    time::Instant,
};

use crossbeam::atomic::AtomicCell;

use crate::{
    config::CONFIG,
    hook_list, lazy_hook,
    shroom_ffi::{self, ztl::zxstr::ZXString8, CiobufferManipulatorDe, CiobufferManipulatorEn, CvecCtrlIsSwimming, CvecCtrlJustJump},
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
    shroom_ffi::clogo_end(this);
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

static SPOOF_IS_SWIMMING: AtomicCell<bool> = AtomicCell::new(false);

static CVEC_CTRL_JUST_JUMP_HOOK: LazyHook<CvecCtrlJustJump> =
    lazy_hook!(shroom_ffi::cvec_ctrl_just_jump, cvec_ctrl_just_jump_hook);

unsafe extern "thiscall" fn cvec_ctrl_just_jump_hook(this: *mut shroom_ffi::CVecCtrl) -> c_int {
    static JUMP_COUNTER: AtomicCell<usize> = AtomicCell::new(0);
    if CVEC_CTRL_JUST_JUMP_HOOK.call(this) == 1 {
        JUMP_COUNTER.store(0);
        return 1;
    }

    if JUMP_COUNTER.load() < CONFIG.get().unwrap().multi_jump.unwrap_or(1) {
        JUMP_COUNTER.fetch_add(1);
        SPOOF_IS_SWIMMING.store(true);
        let res = CVEC_CTRL_JUST_JUMP_HOOK.call(this);
        SPOOF_IS_SWIMMING.store(false);
        res
    } else {
        0
    }
}

static CVEC_CTRL_IS_SWIMMING_HOOK: LazyHook<CvecCtrlIsSwimming> = lazy_hook!(
    shroom_ffi::cvec_ctrl_is_swimming,
    cvec_ctrl_is_swimming_hook
);

unsafe extern "thiscall" fn cvec_ctrl_is_swimming_hook(this: *mut shroom_ffi::CVecCtrl) -> c_int {
    if SPOOF_IS_SWIMMING.load() {
        return 1;
    }

    CVEC_CTRL_IS_SWIMMING_HOOK.call(this)
}

hook_list!(
    JumpHooks,
    CVEC_CTRL_JUST_JUMP_HOOK,
    CVEC_CTRL_IS_SWIMMING_HOOK,
);

static CIOBUFFER_MANIPULATOR_EN_HOOK: LazyHook<CiobufferManipulatorEn> = lazy_hook!(
    shroom_ffi::ciobuffer_manipulator_en,
    ciobuffer_manipulator_en_hook
);

unsafe extern "stdcall" fn ciobuffer_manipulator_en_hook(buf: *mut c_char, ln: c_int) -> c_uchar {
    1
}

static CIOBUFFER_MANIPULATOR_DE_HOOK: LazyHook<CiobufferManipulatorDe> = lazy_hook!(
    shroom_ffi::ciobuffer_manipulator_de,
    ciobuffer_manipulator_de_hook
);

unsafe extern "stdcall" fn ciobuffer_manipulator_de_hook(buf: *mut c_char, ln: c_int) -> c_uchar {
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
        JumpHooks.enable_if(cfg.multi_jump.is_some())?;
        log::info!("Disabling shanda: {}", cfg.disable_shanda);
        ShandaHooks.enable_if(cfg.disable_shanda)?;


        Ok(())
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        let cfg = CONFIG.get().unwrap();
        SKIP_LOGO_HOOK.disable_if(cfg.skip_logo)?;
        CMSGBOX_HOOK.disable_if(cfg.log_msgbox)?;
        CWVS_APP_INITIALIZE_GAME_DATA_HOOK.disable()?;
        JumpHooks.enable_if(cfg.multi_jump.is_some())?;
        ShandaHooks.enable_if(cfg.disable_shanda)?;
        Ok(())
    }
}
