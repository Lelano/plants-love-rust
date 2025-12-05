use super::GpioController;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub struct NoopGpioController {
    blink_on: AtomicBool,
    interval_ms: AtomicU64,
}

impl NoopGpioController {
    pub fn new() -> Self {
        Self {
            blink_on: AtomicBool::new(true),
            interval_ms: AtomicU64::new(1000),
        }
    }
}

impl GpioController for NoopGpioController {
    fn set_blink(&self, on: bool) {
        self.blink_on.store(on, Ordering::Relaxed);
    }

    fn is_blink(&self) -> bool {
        self.blink_on.load(Ordering::Relaxed)
    }
    fn set_interval_ms(&self, ms: u64) {
        self.interval_ms.store(ms, Ordering::Relaxed);
    }
    fn interval_ms(&self) -> u64 {
        self.interval_ms.load(Ordering::Relaxed)
    }
}
