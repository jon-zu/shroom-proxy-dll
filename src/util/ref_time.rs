use std::sync::atomic::{Ordering, AtomicU32};

const TIME_OFFSET: u32 = 100;

pub struct RefTime(AtomicU32);

impl RefTime {
    pub const fn new() -> Self {
        Self(AtomicU32::new(0))
    }
    
    pub fn get_time(&self, real_time: u32) -> u32 {
        let ref_t = self.
            0
            .compare_exchange(0, real_time, Ordering::Acquire, Ordering::Relaxed)
            .err()
            .unwrap_or(0);

        real_time.wrapping_sub(ref_t) + TIME_OFFSET
    }
}