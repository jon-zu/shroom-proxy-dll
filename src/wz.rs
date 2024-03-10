#![allow(non_snake_case)]

use std::{
    ffi::{c_int, c_uint, c_void},
    mem::MaybeUninit,
    ops::Deref,
    ptr::null_mut,
};

use widestring::U16CString;
use windows::core::{
    interface, w, IUnknown, IUnknown_Vtbl, Interface, BSTR, GUID, HRESULT, PCWSTR, VARIANT,
};

use crate::{
    config::CONFIG,
    lazy_hook,
    shroom_ffi::{
        self, bstr_assign,
        wz::{cwvs_app_init_res_man, CwvsAppInitResMan, ResManParam},
        CWvsApp, IWzPackage, IWzSeekableArchive, ZtlBStrT,
    },
    static_lazy_hook,
    util::hooks::{HookModule, LazyHook},
};

const PFN_COM_APIS: usize = 0xc6db54;
const GLOBAL_RES_MAN: usize = 0xc6f434;
const GLOBAL_ROOT_NS: usize = 0xc6f43c;

type IWzArchive = c_void;
type IWzNameSpaceProperty = c_void;

type PcCreateObject = extern "cdecl" fn(PCWSTR, &GUID, *mut *mut c_void, c_uint) -> HRESULT;
type PcSetRootNameSpace = extern "cdecl" fn(*const c_void, c_int);

#[derive(Debug)]
#[repr(C)]
struct PFnComApis {
    pc_create_object: PcCreateObject,
    pc_free_unused_libraries: *const (),
    pc_serialize_object: *const (),
    pc_serialize_string: *const (),
    pc_set_root_name_space: PcSetRootNameSpace,
}

impl PFnComApis {
    pub fn get() -> &'static mut Self {
        let pfn: *mut PFnComApis = unsafe { std::mem::transmute(PFN_COM_APIS) };
        unsafe { pfn.as_mut() }.unwrap()
    }
}

#[interface("57dfe40b-3e20-4dbc-97e8-805a50f381bf")]
unsafe trait IWzResMan: IUnknown {
    fn get_rootNameSpace(&self, root: *const *const IUnknown) -> HRESULT;
    fn put_rootNameSpace(&self, u0: *mut IUnknown) -> HRESULT;
    fn raw_SetResManParam(
        &self,
        param: c_uint,
        retain_time: c_int,
        ns_cache_time: c_int,
    ) -> HRESULT;
    fn raw_CreateObject(&self, name: PCWSTR, u1: *mut *mut IUnknown) -> HRESULT;
    fn raw_GetObject(
        &self,
        uol: PCWSTR,
        param: VARIANT,
        aux: VARIANT,
        out: *mut VARIANT,
    ) -> HRESULT;
    fn raw_SerializeObject(
        &self,
        u0: *mut IWzArchive,
        u1: VARIANT,
        u2: *const *const IUnknown,
    ) -> HRESULT;
    fn raw_FlushCachedObjects(&self, u0: c_int) -> HRESULT;
    fn raw_OverrideObject(&self, u1: PCWSTR, u2: PCWSTR) -> HRESULT;
}

#[interface("2aeeeb36-a4e1-4e2b-8f6f-2e7bdec5c53d")]
unsafe trait IWzNameSpace: IUnknown {
    fn get_item(&self, path: PCWSTR, u1: *mut VARIANT) -> HRESULT;
    fn get_property(&self, u0: PCWSTR, u1: VARIANT, u2: *mut *mut IWzNameSpaceProperty) -> HRESULT;
    fn get_NewEnum(&self, u0: *mut *mut IUnknown) -> HRESULT;
    fn raw_Mount(&self, path: PCWSTR, down: *const c_void, prio: c_int) -> HRESULT;
    fn raw_Unmount(&self, u0: PCWSTR, u1: VARIANT) -> HRESULT;
    fn raw__OnMountEvent(
        &self,
        u0: *mut IWzNameSpace,
        u1: *mut IWzNameSpace,
        u2: PCWSTR,
        u3: c_int,
    ) -> HRESULT;
    fn raw_OnGetLocalObject(
        &self,
        u0: c_int,
        u1: PCWSTR,
        u2: *mut c_int,
        u3: *mut VARIANT,
    ) -> HRESULT;
}

//TODO GUID
#[interface("2aeeeb36-a4e1-4e2b-8f6f-2e7bdec5c531")]
unsafe trait IWzWriteableNameSpace: IUnknown {
    fn get_item(&self, u0: PCWSTR, u1: *mut VARIANT) -> HRESULT;
    fn get_property(&self, u0: PCWSTR, u1: VARIANT, u2: *mut *mut IWzNameSpaceProperty) -> HRESULT;
    fn get_NewEnum(&self, u0: *mut *mut IUnknown) -> HRESULT;
    fn raw_Mount(&self, u0: PCWSTR, u1: *mut IWzNameSpace, u2: c_int) -> HRESULT;
    fn raw_Unmount(&self, u0: PCWSTR, u1: VARIANT) -> HRESULT;
    fn raw_OnMountEvent(
        &self,
        u0: *mut IWzNameSpace,
        u1: *mut IWzNameSpace,
        u2: PCWSTR,
        u3: c_int,
    ) -> HRESULT;
    fn raw_OnGetLocalObject(
        &self,
        u0: c_int,
        u1: PCWSTR,
        u2: *mut c_int,
        u3: *mut VARIANT,
    ) -> HRESULT;
    fn raw_CreateChildNameSpace(&self, u0: PCWSTR, u1: *mut *mut IWzNameSpace) -> HRESULT;
    fn raw_AddObject(&self, u0: PCWSTR, u1: VARIANT, u2: *mut VARIANT) -> HRESULT;
    fn raw_Unlink(&self, u0: PCWSTR) -> HRESULT;
}

#[interface("352d8655-51e4-4668-8ce4-0866e2b6a5b5")]
unsafe trait IWzFileSystem: IWzWriteableNameSpace {
    fn raw_Init(&self, path: PCWSTR) -> HRESULT;
}

fn pcreate_com_ptr<T: Interface>(
    name: PCWSTR,
    obj: *mut ComPtr<T>,
    outer: c_uint,
) -> windows::core::Result<()> {
    (PFnComApis::get().pc_create_object)(name, &T::IID, obj as *mut _ as *mut *mut c_void, outer)
        .ok()
}

#[derive(Debug)]
#[repr(C)]
pub struct ComPtr<T: Interface>(pub *mut T);

impl<T: Interface> ComPtr<T> {
    pub fn create(name: PCWSTR, outer: c_uint) -> windows::core::Result<Self> {
        let mut ptr = ComPtr(null_mut());
        pcreate_com_ptr(name, &mut ptr as *mut _, outer)?;
        Ok(ptr)
    }

    pub fn init(
        name: PCWSTR,
        ptr: &mut MaybeUninit<ComPtr<T>>,
        outer: c_uint,
    ) -> windows::core::Result<&mut Self> {
        unsafe {
            ptr.as_mut_ptr().write(ComPtr(null_mut()));
            pcreate_com_ptr(name, ptr.as_mut_ptr() as *mut ComPtr<T>, outer)?;
            Ok(ptr.assume_init_mut())
        }
    }

    pub unsafe fn as_ref(&self) -> &T {
        T::from_raw_borrowed(std::mem::transmute(&self.0)).unwrap()
    }

    pub fn as_ptr(&self) -> *const c_void {
        self as *const _ as *const c_void
    }
}

impl<T: Interface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }

        if T::UNKNOWN {
            unsafe {
                (self.assume_vtable::<IUnknown>().Release)(self.0 as *mut _);
                self.0 = null_mut();
            }
        }
    }
}

impl<T: Interface> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.as_ref() }
    }
}

static RES_MAN_INIT_HOOK: LazyHook<CwvsAppInitResMan> =
    lazy_hook!(cwvs_app_init_res_man, cwvs_app_init_res_man_hook);

fn get_app_dir() -> Option<String> {
    let dir = std::env::current_exe().ok().unwrap();
    let dir = dir.parent().unwrap();
    Some(dir.to_str()?.to_string().replace('\\', "/"))
}

unsafe fn load_img() -> windows::core::Result<()> {
    let cfg = CONFIG.get().unwrap().wz.as_image().unwrap();
    let g_rm: *mut MaybeUninit<ComPtr<IWzResMan>> = unsafe { std::mem::transmute(GLOBAL_RES_MAN) };
    let g_root: *mut MaybeUninit<ComPtr<IWzNameSpace>> =
        unsafe { std::mem::transmute(GLOBAL_ROOT_NS) };
    let data_dir = cfg.path.as_str();
    let cache_time = cfg.cachte_delay as c_int;
    let retain_time = cfg.retain_delay as c_int;

    let app_dir = match get_app_dir() {
        Some(dir) => dir,
        None => {
            log::info!("Failed to get app dir, using empty dir");
            String::new()
        }
    };

    log::info!("Creating res man");
    let g_rm = ComPtr::init(w!("ResMan"), &mut *g_rm, 0)?;
    g_rm.raw_SetResManParam(
        (ResManParam::AUTO_REPARSE | ResManParam::AUTO_SERIALIZE).bits(),
        retain_time,
        cache_time,
    )
    .ok()?;

    log::info!("Creating root namespace");
    let g_root = ComPtr::init(w!("NameSpace"), &mut *g_root, 0)?;
    (PFnComApis::get().pc_set_root_name_space)(g_root as *const _ as *const c_void, 1);

    log::info!("Creating game fs");
    let fs_game = ComPtr::<IWzFileSystem>::create(w!("NameSpace#FileSystem"), 0)?;
    let data: BSTR = app_dir.into();
    fs_game.raw_Init(PCWSTR(data.as_ptr())).ok()?;
    g_root.raw_Mount(w!("/"), fs_game.as_raw(), 0).ok()?;

    log::info!("Creating data fs");
    let fs_data = ComPtr::<IWzFileSystem>::create(w!("NameSpace#FileSystem"), 0)?;
    let data: BSTR = data_dir.into();
    fs_data.raw_Init(PCWSTR(data.as_ptr())).ok()?;
    g_root.raw_Mount(w!("/"), fs_data.as_raw(), 0).ok()?;

    Ok(())
}

static_lazy_hook!(
    WZ_PACKAGE_HOOK,
    shroom_ffi::IwzPackageInitRef,
    wz_package_hook
);

unsafe extern "thiscall" fn wz_package_hook(
    this: *mut IWzPackage,
    mut key: ZtlBStrT,
    base_uol: ZtlBStrT,
    archive: *mut IWzSeekableArchive,
) -> HRESULT {
    let cfg = CONFIG.get().unwrap();
    if let Some(version) = cfg.wz.as_wz().map(|wz| &wz.version) {
        key.assign_wide(version.as_pcwstr());
    }
    let key_ = key.as_wide().map(|s| s.to_string());
    let base_uol_ = base_uol.as_wide().map(|s| s.to_string());
    log::info!("wz_package_hook: {:?} {:?}", key_, base_uol_);
    WZ_PACKAGE_HOOK.call(this, key, base_uol, archive)
}

static_lazy_hook!(WZ_FS_HOOK, shroom_ffi::IwzFilesystemInitRef, wz_fs_hook);

unsafe extern "thiscall" fn wz_fs_hook(this: *mut c_void, mut path: ZtlBStrT) -> HRESULT {
    let cfg = CONFIG.get().unwrap();
    log::info!("wz fs init: {:?}", path.as_wide().map(|s| s.to_string()));
    if let Some(new_path) = cfg.wz.as_wz().and_then(|wz| wz.path.as_ref()) {
        let current_dir = std::env::current_dir().unwrap();
        let new_path = current_dir.join(new_path);

        let str = U16CString::from_os_str_truncate(new_path);
        log::info!("wz fs new path: {:?}", str.to_string_lossy());
        bstr_assign()(&mut path as *mut _, PCWSTR(str.as_ptr()));
    }
    WZ_FS_HOOK.call(this, path)
}

#[allow(dead_code)]
unsafe extern "thiscall" fn cwvs_app_init_res_man_hook(_app: *const CWvsApp) {
    log::info!("Loading images");
    if let Err(res) = load_img() {
        log::error!("Failed to load img: {:?} - {}", res, res.message());
        unreachable!("Failed to load img");
    }
}

macro_rules! lazy_load_tmpl {
    (
        $tmpl_mod:ident,
        $get_fn:expr,
        $load_fn:expr
    ) => {
        mod $tmpl_mod {
            type Tmpl = std::ffi::c_void;
            $crate::fn_ref!(
                tmpl_get,
                $get_fn,
                unsafe extern "cdecl" fn(id: std::ffi::c_uint) -> *mut Tmpl
            );
            $crate::fn_ref!(tmpl_load, $load_fn, unsafe extern "cdecl" fn());

            pub static GET_HOOK: $crate::util::hooks::LazyHook<TmplGet> =
                $crate::lazy_hook!(tmpl_get, tmpl_get_hook);

            unsafe extern "cdecl" fn tmpl_get_hook(id: std::ffi::c_uint) -> *mut Tmpl {
                static INIT: std::sync::Once = std::sync::Once::new();
                INIT.call_once(|| {
                    log::info!("Lazy Loading {} templates", stringify!($tmpl_mod));
                    LOAD_HOOK.call();
                    log::info!("Lazy Loaded {} templates", stringify!($tmpl_mod));
                });

                GET_HOOK.call(id)
            }

            pub static LOAD_HOOK: $crate::util::hooks::LazyHook<TmplLoad> =
                $crate::lazy_hook!(tmpl_load, tmpl_load_hook);
            unsafe extern "cdecl" fn tmpl_load_hook() {
                /*use std::sync::atomic::AtomicBool;
                static FIRST: AtomicBool = AtomicBool::new(true);
                if FIRST.load(std::sync::atomic::Ordering::Relaxed) {
                    FIRST.store(false, std::sync::atomic::Ordering::Relaxed);
                    log::info!("Skip first {} templates load", stringify!($tmpl_mod));
                    return;
                }

                log::info!("Loading {} templates", stringify!($tmpl_mod));

                LOAD_HOOK.call()*/
                log::info!("Skipping {} templates load", stringify!($tmpl_mod));

                return;
            }
        }
    };
}
// name, get, load

lazy_load_tmpl!(mob_tmpl, 0x6611f0, 0x6611c0);
lazy_load_tmpl!(morph_tmpl, 0x665050, 0x665d80);
lazy_load_tmpl!(taming_mob_tmpl, 0x75b150, 0x75b910);
lazy_load_tmpl!(npc_tmpl, 0x67f1c0, 0x67f190);
lazy_load_tmpl!(pet_tmpl, 0x6a5f90, 0x6aa940);
lazy_load_tmpl!(reactor_tmpl, 0x6d27c0, 0x6aa940);
lazy_load_tmpl!(employee_tmpl, 0x5195d0, 0x519f00);

pub struct WzHooks;

impl HookModule for WzHooks {
    unsafe fn enable(&self) -> anyhow::Result<()> {
        let cfg = CONFIG.get().unwrap();
        WZ_PACKAGE_HOOK.enable_if(cfg.wz.is_wz())?;
        WZ_FS_HOOK.enable_if(cfg.wz.is_wz())?;
        RES_MAN_INIT_HOOK.enable_if(cfg.wz.is_image())?;

        if cfg.lazy_tmpl_loading {
            mob_tmpl::GET_HOOK.enable()?;
            mob_tmpl::LOAD_HOOK.enable()?;

            morph_tmpl::GET_HOOK.enable()?;
            morph_tmpl::LOAD_HOOK.enable()?;

            taming_mob_tmpl::GET_HOOK.enable()?;
            taming_mob_tmpl::LOAD_HOOK.enable()?;

            pet_tmpl::GET_HOOK.enable()?;
            pet_tmpl::LOAD_HOOK.enable()?;

            npc_tmpl::GET_HOOK.enable()?;
            npc_tmpl::LOAD_HOOK.enable()?;

            reactor_tmpl::GET_HOOK.enable()?;
            reactor_tmpl::LOAD_HOOK.enable()?;

            employee_tmpl::GET_HOOK.enable()?;
            employee_tmpl::LOAD_HOOK.enable()?;
        }

        Ok(())
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        let cfg = CONFIG.get().unwrap();
        WZ_PACKAGE_HOOK.disable_if(cfg.wz.is_wz())?;
        WZ_FS_HOOK.disable_if(cfg.wz.is_wz())?;
        RES_MAN_INIT_HOOK.disable_if(cfg.wz.is_image())?;

        Ok(())
    }
}