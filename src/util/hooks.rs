use retour::{GenericDetour, Function};
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
};

#[macro_export]
macro_rules! fn_ref {
    ($name:ident, $addr:expr, $($fn_ty:tt)*) => {
        paste::paste! {
            #[allow(non_upper_case_globals)]
            pub const [<$name _addr>]: *const () = $addr as *const ();
            pub type [<$name:camel>] = $($fn_ty)*;
            #[allow(non_upper_case_globals)]
            pub static $name: std::sync::LazyLock<[<$name:camel>]> = std::sync::LazyLock::new(|| unsafe {
                std::mem::transmute([<$name _addr>])
            });
        }
    };
}

#[macro_export]
macro_rules! fn_ref_hook {
    ($hook_name:ident, $fn_ty:ty) => {
        retour::static_detour! {
            static $hook_name: $fn_ty;
        }
    };
}

pub unsafe fn ms_fn_hook<F: retour::Function + Sized>(addr: usize, detour: F) -> GenericDetour<F> {
    let f: F = std::mem::transmute_copy(&addr);
    GenericDetour::new(f, detour).expect("MS detour")
}

//TODO impl hookable trait for unsafe fns
#[macro_export]
macro_rules! static_ms_fn_hook {
    ($name:ident, $addr:expr, $detour:ident, type $fnty:ident = $($fn_ty:tt)*) => {
        pub type $fnty = $($fn_ty)*;
        static $name: std::sync::LazyLock<retour::GenericDetour<$fnty>> =
            std::sync::LazyLock::new(|| unsafe { $crate::util::hooks::ms_fn_hook::<$fnty>($addr, $detour) });
    };
}

#[macro_export]
macro_rules! static_ms_fn_ref_hook {
    ($hook_name:ident, $fn:ident, $($fn_ty:tt)*, $detour:ident) => {
        static $hook_name: std::sync::LazyLock<retour::GenericDetour<$($fn_ty)*>> =
            std::sync::LazyLock::new(|| unsafe { $crate::util::hooks::ms_fn_hook($fn, $detour) });
    };
}

pub type LazyHook<T> = std::sync::LazyLock<GenericDetour<T>>;

#[macro_export]
macro_rules! lazy_hook {
    ($target:path, $hook:path) => {
        std::sync::LazyLock::new(move || unsafe {
            retour::GenericDetour::new(*$target, $hook).unwrap()
        })
    };
}

pub unsafe fn win32_fn_hook<F: retour::Function + Sized>(
    module: PCWSTR,
    fn_name: PCSTR,
    detour: F,
) -> GenericDetour<F> {
    let handle = GetModuleHandleW(module).expect("Module");
    let proc = GetProcAddress(handle, fn_name);
    let Some(proc) = proc else {
        panic!("Unknown function {fn_name:?} for module: {module:?}");
    };

    let win_fn: F = std::mem::transmute_copy(&proc);
    GenericDetour::new(win_fn, detour).expect("Win32 detour")
}

#[macro_export]
macro_rules! static_win32_fn_hook {
    ($name:ident, $mod:expr, $fn_name:expr, $detour:ident, type $fnty:ident = $($fn_ty:tt)*) => {
        pub type $fnty = $($fn_ty)*;
        static $name: std::sync::LazyLock<GenericDetour<$fnty>> =
            std::sync::LazyLock::new(|| unsafe { $crate::util::hooks::win32_fn_hook::<$fnty>($mod, $fn_name, $detour) });
    };
}


pub trait HookModule {
    unsafe fn enable(&self) -> anyhow::Result<()>;
    unsafe fn disable(&self) -> anyhow::Result<()>;



    unsafe fn enable_if(&self, cond: bool) -> anyhow::Result<()> {
        if cond {
            self.enable()?;
        }
        Ok(())
    }

    unsafe fn disable_if(&self, cond: bool) -> anyhow::Result<()> {
        if cond {
            self.disable()?;
        }
        Ok(())
    }
}

impl<T: Function>  HookModule for GenericDetour<T> {
    unsafe fn enable(&self) -> anyhow::Result<()> {
        self.enable()?;
        Ok(())
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        self.disable()?;
        Ok(())
    }
}
