use std::{
    ffi::{c_char, c_int, c_uchar, c_void},
    time::Instant,
};


use crate::{
    config::CONFIG,
    hook_list, lazy_hook,
    shroom_ffi::{
        self, ztl::zxstr::ZXString8, CiobufferManipulatorDe, CiobufferManipulatorEn
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

/* 
unsafe extern "fastcall" fn encode_dmg(this: *mut COutPacket, crit: u32, dmg: u32) {
    coutpacket_encode4()(this, dmg);
    coutpacket_encode4()(this, crit);
}


#[naked]
pub(crate) unsafe extern "thiscall" fn melee_encode4_hook(this: *mut COutPacket, val: c_int) {
    unsafe {
        std::arch::asm!(
            // push the crit field
            // edx is reserved during call
            "mov edx, dword [ebp-0xbf8]",
            "mov edx, dword [edx+eax*4+0x54]",
            "jmp encode_dmg",
            options(noreturn)
        );
    }
}


#[naked]
pub(crate) unsafe extern "thiscall" fn shoot_encode4_hook(this: *mut COutPacket, val: c_int) {
    unsafe {
        std::arch::asm!(
            // push the crit field
            // edx is reserved during call
            "mov edx, dword [ebp-0x1050]",
            "mov edx, dword [edx+eax*4+0x54]",
            "jmp encode_dmg",
            options(noreturn)
        );
    }
}

#[naked]
pub(crate) unsafe extern "thiscall" fn magic_encode4_hook(this: *mut COutPacket, val: c_int) {
    unsafe {
        std::arch::asm!(
            // push the crit field
            // edx is reserved during call
            "mov edx, dword [ebp-0x1538]",
            "mov edx, dword [edx+eax*4+0x54]",
            "jmp encode_dmg",
            options(noreturn)
        );
    }
}


#[naked]
pub(crate) unsafe extern "thiscall" fn body_encode4_hook(this: *mut COutPacket, val: c_int) {
    unsafe {
        std::arch::asm!(
            // push the crit field
            // edx is reserved during call
            "mov edx, dword [ebp-0x1768]",
            "mov edx, dword [edx+eax*4+0x54]",
            "jmp encode_dmg",
            options(noreturn)
        );
    }
}

#[naked]
pub(crate) unsafe extern "thiscall" fn meso_explosion_encode4_hook(this: *mut COutPacket, val: c_int) {
    unsafe {
        std::arch::asm!(
            // push the crit field
            // edx is reserved during call
            "mov eax, dword [ebp-0xb30]",
            "mov edx, dword [ebp-0xb20]",
            "mov edx, dword [edx+eax*4+0x54]",
            "jmp encode_dmg",
            options(noreturn)
        );
    }
}
*/

pub struct ShroomHooks;

impl HookModule for ShroomHooks {
    unsafe fn enable(&self) -> anyhow::Result<()> {
        let cfg = CONFIG.get().unwrap();
        SKIP_LOGO_HOOK.enable_if(cfg.skip_logo)?;
        CMSGBOX_HOOK.enable_if(cfg.log_msgbox)?;
        CWVS_APP_INITIALIZE_GAME_DATA_HOOK.enable()?;
        ShandaHooks.enable_if(cfg.disable_shanda)?;

        Ok(())
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        let cfg = CONFIG.get().unwrap();
        SKIP_LOGO_HOOK.disable_if(cfg.skip_logo)?;
        CMSGBOX_HOOK.disable_if(cfg.log_msgbox)?;
        CWVS_APP_INITIALIZE_GAME_DATA_HOOK.disable()?;
        ShandaHooks.disable_if(cfg.disable_shanda)?;

        Ok(())
    }
}
