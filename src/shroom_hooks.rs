use std::ffi::c_void;

use crate::{
    lazy_hook, shroom_ffi,
    util::hooks::{HookModule, LazyHook},
};

static SKIP_LOGO_HOOK: LazyHook<shroom_ffi::ClogoInit> =
    lazy_hook!(shroom_ffi::clogo_init, clogo_init_hook);
unsafe extern "thiscall" fn clogo_init_hook(this: *mut shroom_ffi::CLogo, _param: *const c_void) {
    shroom_ffi::clogo_end(this);
}

pub struct ShroomHooks;

impl HookModule for ShroomHooks {
    unsafe fn enable(&self) -> anyhow::Result<()> {
        SKIP_LOGO_HOOK.enable()?;
        Ok(())
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        SKIP_LOGO_HOOK.disable()?;
        Ok(())
    }
}
