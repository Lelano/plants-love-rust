use super::GpioController;
use chrono::{Local, Timelike, Datelike, Weekday};
use rppal::gpio::Gpio;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct GpioSchedule {
    // Vec of (start, end) in 24h HHMM (e.g., 930, 1745)
    pub schedule: HashMap<Weekday, Vec<(u16, u16)>>,
}

pub struct ScheduleRppalGpioController {
    enabled: Arc<AtomicBool>,
    interval_ms: Arc<AtomicU64>, // kept for trait compatibility; not used
    _thread: thread::JoinHandle<()>,
}

impl ScheduleRppalGpioController {
    pub fn new(gpio_pin: u8, invert: bool, sched: GpioSchedule) -> Self {
        let enabled = Arc::new(AtomicBool::new(true));
        let interval_ms = Arc::new(AtomicU64::new(1000));

        let enabled_t = Arc::clone(&enabled);
        let schedule_t = sched.schedule.clone();

        let handle = thread::spawn(move || {
            println!("[gpio-sched] pin={} invert={}", gpio_pin, invert);
            let gpio = match Gpio::new() {
                Ok(g) => g,
                Err(_) => return,
            };
            let mut pin = match gpio.get(gpio_pin).and_then(|p| Ok(p.into_output())) {
                Ok(p) => p,
                Err(_) => return,
            };

            loop {
                let now = Local::now();
                let wd = now.weekday();
                let hhmm: u16 = (now.time().hour() as u16) * 100 + (now.time().minute() as u16);

                let mut on = false;
                if enabled_t.load(Ordering::Relaxed) {
                    if let Some(ranges) = schedule_t.get(&wd) {
                        for (start, end) in ranges {
                            if *start <= hhmm && hhmm < *end { on = true; break; }
                        }
                    }
                }

                let high = if invert { !on } else { on };
                if high { pin.set_high(); } else { pin.set_low(); }

                thread::sleep(Duration::from_millis(500));
            }
        });

        Self { enabled, interval_ms, _thread: handle }
    }
}

impl GpioController for ScheduleRppalGpioController {
    fn set_blink(&self, on: bool) {
        self.enabled.store(on, Ordering::Relaxed);
    }
    fn is_blink(&self) -> bool { // returns schedule enabled
        self.enabled.load(Ordering::Relaxed)
    }
    fn set_interval_ms(&self, ms: u64) { // not used by scheduler
        self.interval_ms.store(ms, Ordering::Relaxed);
    }
    fn interval_ms(&self) -> u64 { // not used by scheduler
        self.interval_ms.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ignored: requires GPIO hardware. Ensures API compiles with `--features gpio`.
    #[test]
    #[ignore]
    fn construct_schedule_controller() {
        let mut m: HashMap<Weekday, Vec<(u16, u16)>> = HashMap::new();
        m.insert(Weekday::Mon, vec![(900, 1700)]);
        let sched = GpioSchedule { schedule: m };
        let ctl = ScheduleRppalGpioController::new(27, false, sched);
        let _ = ctl.is_blink();
    }
}
