use super::GpioController;
use rppal::gpio::Gpio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct RppalGpioController {
    blink_on: Arc<AtomicBool>,
    interval_ms: Arc<AtomicU64>,
    _thread: thread::JoinHandle<()>,
}

impl RppalGpioController {
    pub fn new(gpio_pin: u8, invert: bool) -> Self {
        let blink_on = Arc::new(AtomicBool::new(true));
        let interval_ms = Arc::new(AtomicU64::new(1000));

        let blink_on_t = Arc::clone(&blink_on);
        let interval_t = Arc::clone(&interval_ms);

        let handle = thread::spawn(move || {
            println!("[gpio] thread start pin={} invert={}", gpio_pin, invert);
            // Prepare GPIO pin
            let gpio = match Gpio::new() {
                Ok(g) => g,
                Err(_) => return,
            };
            let mut pin = match gpio.get(gpio_pin).and_then(|p| Ok(p.into_output())) {
                Ok(p) => p,
                Err(_) => return,
            };

            let mut last = Instant::now();
            let mut state = false;
            loop {
                let iv = Duration::from_millis(interval_t.load(Ordering::Relaxed));
                if blink_on_t.load(Ordering::Relaxed) {
                    if last.elapsed() >= iv {
                        state = !state;
                        let high = if invert { !state } else { state };
                        if high { pin.set_high(); } else { pin.set_low(); }
                        last = Instant::now();
                    }
                } else if state {
                    let high = if invert { true } else { false };
                    if high { pin.set_high(); } else { pin.set_low(); }
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
