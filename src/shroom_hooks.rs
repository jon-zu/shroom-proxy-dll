use std::{ffi::c_void, time::Instant};

use crate::{
    config::CONFIG,
    lazy_hook,
    shroom_ffi::{self, ztl::zxstr::ZXString8},
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

/*
Sampling:
let start = Instant::now();
    log::info!("Loading game data...");
    let sample = CpuSampler::profile(|| CWVS_APP_INITIALIZE_GAME_DATA_HOOK.call(this));
    let elapsed = start.elapsed();

    let modules = AddressModuleMapper::new().unwrap();
    for sample in sample.iter().take(100) {
        if let Some(module) = modules.get_module(sample.0) {
            log::info!(
                "{:#X}: {} ({})",
                sample.0,
                sample.1 .1 * 100.,
                AddressModuleMapper::get_module_name(module).to_str().unwrap()
            );
        } else {
            log::info!("{:#X}: {}", sample.0, sample.1 .1 * 100.);
        }
        log::info!("{:#X}: {}", sample.0, sample.1 .1 * 100.);
    }

    log::info!("cwvs_app_initialize_game_data took {:?}", elapsed); */
unsafe extern "thiscall" fn cwvs_app_initialize_game_data_hook(this: *mut shroom_ffi::CWvsApp) {
    let start = Instant::now();
    CWVS_APP_INITIALIZE_GAME_DATA_HOOK.call(this);
    log::info!("cwvs_app_initialize_game_data took {:?}", start.elapsed());
}

pub struct ShroomHooks;

impl HookModule for ShroomHooks {
    unsafe fn enable(&self) -> anyhow::Result<()> {
        let cfg = CONFIG.get().unwrap();
        SKIP_LOGO_HOOK.enable_if(cfg.skip_logo)?;
        CMSGBOX_HOOK.enable_if(cfg.log_msgbox)?;
        CWVS_APP_INITIALIZE_GAME_DATA_HOOK.enable()?;
        Ok(())
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        let cfg = CONFIG.get().unwrap();
        SKIP_LOGO_HOOK.disable_if(cfg.skip_logo)?;
        CMSGBOX_HOOK.disable_if(cfg.log_msgbox)?;
        CWVS_APP_INITIALIZE_GAME_DATA_HOOK.disable()?;
        Ok(())
    }
}
