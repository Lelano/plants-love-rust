// Entry point that wires together UI, config, and GPIO controller.

mod ui;
mod config;
mod gpio;

use crate::config::load_config;
use crate::gpio::new_controller;
#[cfg(feature = "gpio")]
use chrono::Weekday;
#[cfg(feature = "gpio")]
use crate::gpio::ScheduleRppalGpioController;
#[cfg(feature = "gpio")]
use crate::gpio::GpioSchedule;
#[cfg(feature = "gpio")]
use std::collections::HashMap;

fn main() {
    // Load persisted configuration
    let cfg = load_config();

    // Construct GPIO controller (real or stub depending on features)
    println!(
        "[startup] pin={} invert={} blink={} iv={}ms",
        cfg.gpio_pin, cfg.invert, cfg.blink_on, cfg.interval_ms
    );
    // If a schedule is provided in config, start a schedule controller on cfg.schedule_pin
    #[cfg(feature = "gpio")]
    {
        if let Some(s) = build_schedule(cfg.schedule.clone()) {
            let _schedule_ctl = ScheduleRppalGpioController::new(cfg.schedule_pin, cfg.invert, s);
            let _ = &_schedule_ctl;
            println!("[startup] schedule active on GPIO {}", cfg.schedule_pin);
        }
    }

    // Interval controller on GPIO 17 for the TUI
    let controller = new_controller(cfg.gpio_pin, cfg.invert, None);
    controller.set_blink(cfg.blink_on);
    controller.set_interval_ms(cfg.interval_ms);

    // Run the terminal UI only
    if let Err(e) = ui::run(controller, cfg) {
        eprintln!("TUI error: {e}");
    }
}

// Build schedule from config: String day names -> Weekday map
#[cfg(feature = "gpio")]
fn build_schedule(src: Option<HashMap<String, Vec<(u16, u16)>>>) -> Option<GpioSchedule> {
    let mut map: HashMap<Weekday, Vec<(u16, u16)>> = HashMap::new();
    let Some(srcmap) = src else { return None; };
    for (k, v) in srcmap.into_iter() {
        if let Some(day) = parse_weekday(&k) {
            let normalized = normalize_ranges(&k, v);
            if !normalized.is_empty() {
                map.entry(day).or_default().extend(normalized);
            }
        }
    }
    if map.is_empty() { None } else { Some(GpioSchedule { schedule: map }) }
}

#[cfg(feature = "gpio")]
fn parse_weekday(s: &str) -> Option<Weekday> {
    let t = s.trim().to_lowercase();
    match t.as_str() {
        "mon" | "monday" => Some(Weekday::Mon),
        "tue" | "tues" | "tuesday" => Some(Weekday::Tue),
        "wed" | "weds" | "wednesday" => Some(Weekday::Wed),
        "thu" | "thur" | "thurs" | "thursday" => Some(Weekday::Thu),
        "fri" | "friday" => Some(Weekday::Fri),
        "sat" | "saturday" => Some(Weekday::Sat),
        "sun" | "sunday" => Some(Weekday::Sun),
        _ => None,
    }
}

#[cfg(feature = "gpio")]
fn normalize_ranges(day_key: &str, ranges: Vec<(u16, u16)>) -> Vec<(u16, u16)> {
    // Filter invalid HHMM and start/end, then sort and merge overlaps/adjacent.
    let mut valid: Vec<(u16, u16)> = ranges
        .into_iter()
        .filter(|(s, e)| {
            let ok = is_hhmm(*s) && is_hhmm(*e) && s < e;
            if !ok {
                println!("[schedule] drop invalid {:?} for {}", (s, e), day_key);
            }
            ok
        })
        .collect();

    valid.sort_by_key(|(s, _)| *s);
    let mut out: Vec<(u16, u16)> = Vec::new();
    for (s, e) in valid {
        if let Some((_last_s, last_e)) = out.last_mut() {
            if s <= *last_e { // overlap or touch; merge
                if e > *last_e { *last_e = e; }
                continue;
            }
        }
        out.push((s, e));
    }
    out
}

#[cfg(feature = "gpio")]
fn is_hhmm(v: u16) -> bool {
    let hh = v / 100;
    let mm = v % 100;
    hh < 24 && mm < 60
}