use super::GpioController;
use rppal::gpio::Gpio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

pub struct RppalGpioController {
    blink_on: AtomicBool,
    interval_ms: AtomicU64,
    _thread: thread::JoinHandle<()>,
}

impl RppalGpioController {
    pub fn new() -> Self {
        let blink_on = AtomicBool::new(true);
        let interval_ms = AtomicU64::new(1000);

        let blink_on_ref = &blink_on as *const AtomicBool as usize;
        let interval_ref = &interval_ms as *const AtomicU64 as usize;

        let handle = thread::spawn(move || {
            // reconstruct references
            let blink_on: &AtomicBool = unsafe { &*(blink_on_ref as *const AtomicBool) };
            let interval_ms: &AtomicU64 = unsafe { &*(interval_ref as *const AtomicU64) };

            // Prepare GPIO17
            let gpio = match Gpio::new() {
                Ok(g) => g,
                Err(_) => return,
            };
            let mut pin = match gpio.get(17).and_then(|p| Ok(p.into_output())) {
                Ok(p) => p,
                Err(_) => return,
            };

            let mut last = Instant::now();
            let mut state = false;
            loop {
                let iv = Duration::from_millis(interval_ms.load(Ordering::Relaxed));
                if blink_on.load(Ordering::Relaxed) {
                    if last.elapsed() >= iv {
                        state = !state;
                        if state { pin.set_high(); } else { pin.set_low(); }
                        last = Instant::now();
                    }
                } else if state {
                    pin.set_low();
                    state = false;
                }
                thread::sleep(Duration::from_millis(10));
            }
        });

        Self { blink_on, interval_ms, _thread: handle }
    }
}

impl GpioController for RppalGpioController {
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
