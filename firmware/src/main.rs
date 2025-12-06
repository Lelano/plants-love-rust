// Entry point that wires together UI, config, and GPIO controller.

mod ui;
mod config;
mod gpio;

use crate::config::load_config;
use crate::gpio::{new_controller, GpioSchedule};
use std::collections::HashMap;
#[cfg(feature = "gpio")]
use chrono::Weekday;

fn main() {
    // Load persisted configuration
    let cfg = load_config();

    // Construct GPIO controller (real or stub depending on features)
    println!(
        "[startup] pin={} invert={} blink={} iv={}ms",
        cfg.gpio_pin, cfg.invert, cfg.blink_on, cfg.interval_ms
    );
    let sched = build_schedule(cfg.schedule.clone());
    let controller = new_controller(cfg.gpio_pin, cfg.invert, sched);
    controller.set_blink(cfg.blink_on);
    controller.set_interval_ms(cfg.interval_ms);

    // Run the terminal UI only
    if let Err(e) = ui::run(controller, cfg) {
        eprintln!("TUI error: {e}");
    }
}

#[cfg(feature = "gpio")]
fn build_schedule(src: Option<HashMap<String, Vec<(u16, u16)>>>) -> Option<GpioSchedule> {
    let mut map: HashMap<Weekday, Vec<(u16, u16)>> = HashMap::new();
    let Some(srcmap) = src else { return None; };
    for (k, v) in srcmap.into_iter() {
        if let Some(day) = parse_weekday(&k) {
            map.entry(day).or_default().extend(v);
        }
    }
    if map.is_empty() { None } else { Some(GpioSchedule { schedule: map }) }
}
#[cfg(not(feature = "gpio"))]
fn build_schedule(_src: Option<HashMap<String, Vec<(u16, u16)>>>) -> Option<GpioSchedule> { None }

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