use retour::{Function, GenericDetour};
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
};

pub trait FnRef {
    type Fn: Function;
    fn get_fn() -> Self::Fn;
}

#[macro_export]
macro_rules! fn_ref {
    ($name:ident, $addr:expr, $($fn_ty:tt)*) => {
        paste::paste! {
            #[allow(non_upper_case_globals)]
            pub const [<$name _addr>]: *const () = $addr as *const ();
            pub type [<$name:camel>] = $($fn_ty)*;
            #[allow(non_upper_case_globals)]
            pub fn $name() -> [<$name:camel>] {
                unsafe { std::mem::transmute([<$name _addr>]) }
            }

            pub struct [<$name:camel Ref>];
            impl $crate::util::hooks::FnRef for [<$name:camel Ref>] {
                type Fn = [<$name:camel>];
                fn get_fn() -> Self::Fn {
                    $name()
                }
            }
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
            retour::GenericDetour::new($target(), $hook).unwrap()
        })
    };
}

#[macro_export]
macro_rules! static_lazy_hook {
    ($name:ident, $target_ty:ty, $hook:path) => {
        static $name: $crate::util::hooks::LazyHook<
            <$target_ty as $crate::util::hooks::FnRef>::Fn,
        > = std::sync::LazyLock::new(move || unsafe {
            use $crate::util::hooks::FnRef;
            retour::GenericDetour::new(<$target_ty>::get_fn(), $hook).unwrap()
        });
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

#[macro_export]
macro_rules! hook_list {
    ($name:ident, $($hook:ident,)+) => {
        pub struct $name;

        impl $crate::util::hooks::HookModule for $name {
            unsafe fn enable(&self) -> anyhow::Result<()> {
                $($hook.enable()?;)*
                Ok(())
            }

            unsafe fn disable(&self) -> anyhow::Result<()> {
                $($hook.disable()?;)*
                Ok(())
            }
        }
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

impl<T: Function> HookModule for GenericDetour<T> {
    unsafe fn enable(&self) -> anyhow::Result<()> {
        self.enable()?;
        Ok(())
    }

    unsafe fn disable(&self) -> anyhow::Result<()> {
        self.disable()?;
        Ok(())
    }
}
