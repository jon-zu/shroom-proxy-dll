use std::{
    collections::HashMap,
    ffi::CString,
    num::Saturating,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use windows::Win32::{
    Foundation::{DuplicateHandle, DUPLICATE_SAME_ACCESS, HANDLE, HMODULE},
    System::{
        Diagnostics::Debug::{GetThreadContext, CONTEXT, CONTEXT_CONTROL_X86},
        ProcessStatus::{EnumProcessModules, GetModuleBaseNameA, GetModuleInformation, MODULEINFO},
        Threading::{GetCurrentProcess, GetCurrentThread, Sleep},
    },
};

pub struct AddressModuleMapper {
    modules: Vec<(usize, usize, HMODULE)>,
}

impl AddressModuleMapper {
    pub fn new() -> windows::core::Result<Self> {
        let mut modules = [HMODULE(0); 1024];
        let mut needed = 0u32;
        unsafe {
            EnumProcessModules(
                GetCurrentProcess(),
                &mut modules as *mut _ as *mut _,
                std::mem::size_of_val(&modules) as u32,
                &mut needed as *mut _,
            )
        }?;

        let mut mods = Vec::new();
        for module in modules
            .into_iter()
            .take(needed as usize / std::mem::size_of::<HMODULE>())
        {
            let mut mod_info: MODULEINFO = unsafe { std::mem::zeroed() };
            unsafe {
                GetModuleInformation(
                    GetCurrentProcess(),
                    module,
                    &mut mod_info as *mut _,
                    std::mem::size_of_val(&mod_info) as u32,
                )
            }?;

            let base = mod_info.lpBaseOfDll as usize;
            mods.push((base, mod_info.SizeOfImage as usize, module));
        }

        Ok(Self { modules: mods })
    }

    pub fn get_module(&self, addr: usize) -> Option<HMODULE> {
        self.modules
            .iter()
            .find(|(base, size, _)| addr >= *base && addr < *base + *size)
            .map(|(_, _, module)| *module)
    }

    pub fn get_module_name(module: HMODULE) -> CString {
        let mut v = vec![0u8; 1024];
        let n = unsafe { GetModuleBaseNameA(GetCurrentProcess(), module, &mut v) };
        v.truncate(n as usize + 1);

        CString::from_vec_with_nul(v).unwrap()
    }
}

pub struct CpuSampler {
    sampling_thread: HANDLE,
    sample: Mutex<HashMap<usize, Saturating<usize>>>,
    quit: AtomicBool,
    running: AtomicBool,
    samples: AtomicUsize,
}

impl CpuSampler {
    fn run(&self) {
        let mut sample = self.sample.try_lock().unwrap();
        const N: usize = 1;
        let mut ctx: CONTEXT = unsafe { std::mem::zeroed() };
        ctx.ContextFlags = CONTEXT_CONTROL_X86;

        const G: usize = 2048;

        log::info!("Starting sampling");
        let mut samples = Saturating(0usize);
        self.running.store(true, Ordering::SeqCst);
        while !self.quit.load(Ordering::SeqCst) {
            for _ in 0..N {
                ctx = unsafe { std::mem::zeroed() };
                ctx.ContextFlags = CONTEXT_CONTROL_X86;
                let res = unsafe { GetThreadContext(self.sampling_thread, &mut ctx as *mut _) };
                if res.is_err() {
                    log::error!("Failed to get thread context");
                    std::thread::sleep(Duration::from_millis(600));
                    break;
                }
                let eip = (ctx.Eip as usize / G) * G;
                *sample.entry(eip).or_default() += 1;
                samples += 1;
            }
        }
        self.samples.store(samples.0, Ordering::SeqCst);
    }

    pub fn profile(f: impl FnOnce() -> ()) -> Vec<(usize, (usize, f64))> {
        let mut thread_handle = HANDLE(0);
        let _ = unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                GetCurrentThread(),
                GetCurrentProcess(),
                &mut thread_handle as *mut _,
                0,
                true,
                DUPLICATE_SAME_ACCESS,
            )
        };
        let sampler = Arc::new(Self {
            sampling_thread: thread_handle,
            sample: Mutex::new(HashMap::new()),
            quit: AtomicBool::new(false),
            running: AtomicBool::new(false),
            samples: AtomicUsize::new(0),
        });
        let sampler_move = sampler.clone();
        let _ = std::thread::spawn(move || sampler_move.run());

        while !sampler.running.load(Ordering::SeqCst) {
            unsafe {
                Sleep(1);
            }
        }
        f();
        sampler.quit.store(true, Ordering::SeqCst);
        let sample = sampler.sample.lock().unwrap();

        let total = sampler.samples.load(Ordering::SeqCst) as f64;
        let mut sample = sample
            .iter()
            .map(|(k, v)| (*k, (v.0, v.0 as f64 / total)))
            .collect::<Vec<_>>();
        sample.sort_by_key(|(_, v)| v.0);
        sample.reverse();
        sample
    }
}
