use std::{
    ffi::{c_int, c_void},
    sync::atomic::AtomicPtr,
};

use crate::{
    config::{AutoLoginData, CONFIG},
    hook_list, shroom_ffi::{self, CLogin, CloginInitRef, CloginOnRecommendWorldMessageRef, CuiavatarSelectCharacterRef},
    static_lazy_hook,
};

//use crate::util::hooks::FnRef;

fn get_auto_login() -> &'static Option<AutoLoginData> {
    &CONFIG.get().unwrap().auto_login_data
}

static CLOGIN_INSTANCE: AtomicPtr<CLogin> = AtomicPtr::new(std::ptr::null_mut());

static_lazy_hook!(INIT_HOOK, CloginInitRef, clogin_init_hook);
unsafe extern "thiscall" fn clogin_init_hook(
    this: *const shroom_ffi::CLogin,
    param: *const c_void,
) {
    INIT_HOOK.call(this, param);
    if let Some(auto_login) = get_auto_login() {
        shroom_ffi::clogin_send_check_password_packet()(
            this,
            auto_login.username.as_pcstr(),
            auto_login.password.as_pcstr(),
        );
    }
}

static_lazy_hook!(
    WORLD_MSG_HOOK,
    CloginOnRecommendWorldMessageRef,
    world_msg_hook
);

unsafe extern "thiscall" fn world_msg_hook(
    this: *const shroom_ffi::CLogin,
    pkt: *const c_void,
) {
    log::info!("On recommended world");
    WORLD_MSG_HOOK.call(this, pkt);
    CLOGIN_INSTANCE.store(this as *mut CLogin, std::sync::atomic::Ordering::SeqCst);
    if let Some((world, channel)) = get_auto_login()
        .as_ref()
        .and_then(|a| a.get_world_channel())
    {
        log::info!("Selecting world: {world} - channel: {channel}");
        shroom_ffi::clogin_send_login_packet()(this, world as i32, channel as i32);
    }
}

static_lazy_hook!(
    SELECT_CHAR_HOOK,
    CuiavatarSelectCharacterRef,
    select_char_hook
);

unsafe extern "thiscall" fn select_char_hook(
    this: *const shroom_ffi::CUIAvatar,
    idx: c_int,
) {
    if let Some(char_index) = get_auto_login().as_ref().and_then(|a| a.char_index) {
        SELECT_CHAR_HOOK.call(this, char_index as i32);
        let login_instance = CLOGIN_INSTANCE.load(std::sync::atomic::Ordering::SeqCst);
        shroom_ffi::clogin_send_select_character_packet()(login_instance);
    } else {
        SELECT_CHAR_HOOK.call(this, idx);
    }
}

hook_list!(
    LoginHooks,
    INIT_HOOK,
    WORLD_MSG_HOOK,
    SELECT_CHAR_HOOK,
);
