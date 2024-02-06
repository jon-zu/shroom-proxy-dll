use std::{
    ffi::{c_int, c_void},
    sync::atomic::AtomicPtr,
};

use crate::{
    config::{AutoLoginData, CONFIG},
    lazy_hook,
    shroom_ffi::{self, CLogin},
    util::hooks::{HookModule, LazyHook},
};

fn get_auto_login() -> &'static Option<AutoLoginData> {
    &CONFIG.get().unwrap().auto_login_data
}

static CLOGIN_INSTANCE: AtomicPtr<CLogin> = AtomicPtr::new(std::ptr::null_mut());

static CLOGIN_INIT_HOOK: LazyHook<shroom_ffi::CloginInit> =
    lazy_hook!(shroom_ffi::clogin_init, clogin_init_hook);

unsafe extern "thiscall" fn clogin_init_hook(
    this: *const shroom_ffi::CLogin,
    param: *const c_void,
) {
    CLOGIN_INIT_HOOK.call(this, param);
    if let Some(auto_login) = get_auto_login() {
        shroom_ffi::clogin_send_check_password_packet(
            this,
            windows::core::PCSTR(auto_login.username.as_ptr() as *const u8),
            windows::core::PCSTR(auto_login.password.as_ptr() as *const u8),
        );
    }
}

static CLOGIN_ON_RECOMMENDED_WORLD_MESSAGE_HOOK: LazyHook<
    shroom_ffi::CloginOnRecommendWorldMessage,
> = lazy_hook!(
    shroom_ffi::clogin_on_recommend_world_message,
    clogin_on_recommend_world_message_hook
);

unsafe extern "thiscall" fn clogin_on_recommend_world_message_hook(
    this: *const shroom_ffi::CLogin,
    pkt: *const c_void,
) {
    log::info!("On recommended world");
    CLOGIN_ON_RECOMMENDED_WORLD_MESSAGE_HOOK.call(this, pkt);
    if let Some(auto_login) = get_auto_login() {
        log::info!("Selecting world: {} - channel: {}", auto_login.world, auto_login.channel);
        shroom_ffi::clogin_send_login_packet(
            this,
            auto_login.world as i32,
            auto_login.channel as i32,
        );
        CLOGIN_INSTANCE.store(this as *mut CLogin, std::sync::atomic::Ordering::SeqCst);
    }
}

static CUIAVATAR_SELECT_CHARACTER_HOOK: LazyHook<shroom_ffi::CuiavatarSelectCharacter> = lazy_hook!(
    shroom_ffi::cuiavatar_select_character,
    cuiavatar_select_character_hook
);

unsafe extern "thiscall" fn cuiavatar_select_character_hook(
    this: *const shroom_ffi::CUIAvatar,
    idx: c_int,
) {
    if let Some(auto_login) = get_auto_login() {
        CUIAVATAR_SELECT_CHARACTER_HOOK.call(this, auto_login.char_index as i32);
        let login_instance = CLOGIN_INSTANCE.load(std::sync::atomic::Ordering::SeqCst);
        shroom_ffi::clogin_send_select_character_packet(login_instance);
    } else {
        CUIAVATAR_SELECT_CHARACTER_HOOK.call(this, idx);
    }
}

pub struct LoginHooks;

impl HookModule for LoginHooks {
    unsafe fn enable(&self) -> anyhow::Result<()> {
        CLOGIN_INIT_HOOK.enable()?;
        CLOGIN_ON_RECOMMENDED_WORLD_MESSAGE_HOOK.enable()?;
        CUIAVATAR_SELECT_CHARACTER_HOOK.enable()?;
        Ok(())
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        CLOGIN_INIT_HOOK.disable()?;
        CLOGIN_ON_RECOMMENDED_WORLD_MESSAGE_HOOK.disable()?;
        CUIAVATAR_SELECT_CHARACTER_HOOK.disable()?;
        Ok(())
    }
}