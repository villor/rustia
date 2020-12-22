use atomic_counter::{AtomicCounter, RelaxedCounter};

pub struct AtomicU32Counter(RelaxedCounter);

impl AtomicU32Counter {
    pub fn new(initial_count: u32) -> Self {
        Self(RelaxedCounter::new(initial_count as usize))
    }

    pub fn inc(&self) -> u32 {
        self.0.inc() as u32
    }
}
