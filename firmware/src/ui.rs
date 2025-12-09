use crate::config::{save_config, AppConfig};
use crate::gpio::GpioController;
use crate::analog::Ads1115;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal;
use crossterm::{cursor, execute, style, terminal::ClearType};
use std::error::Error;
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Compact TUI: fixed-position lines and short text
const MAX_LINE_CHARS: usize = 30;

fn clip_line(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars().take(MAX_LINE_CHARS) {
        out.push(ch);
    }
    out
}

fn draw_lines(out: &mut impl Write, lines: &[String]) -> std::io::Result<()> {
    for (i, l) in lines.iter().enumerate() {
        let y = i as u16;
        let text = clip_line(l);
        execute!(
            out,
            cursor::MoveTo(0, y),
            terminal::Clear(ClearType::CurrentLine),
            style::Print(text),
        )?;
    }
    Ok(())
}

#[inline]
fn clamp_interval(ms: u64) -> u64 {
    ms.clamp(50, 10_000)
}

fn toggle_blink(ctl: &dyn GpioController, cfg: &mut AppConfig) {
    let new = !ctl.is_blink();
    ctl.set_blink(new);
    cfg.blink_on = new;
    let _ = save_config(cfg);
}

fn adjust_interval(ctl: &dyn GpioController, cfg: &mut AppConfig, delta: i64) {
    let cur = ctl.interval_ms() as i64;
    let next = (cur + delta).clamp(50, 10_000) as u64;
    let next = clamp_interval(next);
    ctl.set_interval_ms(next);
    cfg.interval_ms = next;
    let _ = save_config(cfg);
}

pub fn run(
    ctl: Arc<dyn GpioController + Send + Sync>,
    mut cfg: AppConfig,
    sensor: Option<Arc<Mutex<Ads1115>>>,
) -> Result<(), Box<dyn Error>> {
    let mut out = stdout();
    terminal::enable_raw_mode()?;
    execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut running = true;
    loop {
        draw_ui(&ctl, cfg.gpio_pin, cfg.invert, &sensor, &cfg)?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => running = false,
                    KeyCode::Char('b') => toggle_blink(ctl.as_ref(), &mut cfg),
                    KeyCode::Char('+') => adjust_interval(ctl.as_ref(), &mut cfg, -100),
                    KeyCode::Char('-') => adjust_interval(ctl.as_ref(), &mut cfg, 100),
                    KeyCode::Char('d') => calibrate_dry(&sensor, &mut cfg),
                    KeyCode::Char('w') => calibrate_wet(&sensor, &mut cfg),
                    _ => {}
                }
            }
        }

        if !running { break; }
    }

    execute!(out, terminal::LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn calibrate_dry(sensor: &Option<Arc<Mutex<Ads1115>>>, cfg: &mut AppConfig) {
    if let Some(sensor_arc) = sensor {
        if let Ok(mut sensor_lock) = sensor_arc.try_lock() {
            if let Ok(raw) = sensor_lock.read_moisture_sensor() {
                cfg.moisture_dry_value = Some(raw);
                let _ = save_config(cfg);
            }
        }
    }
}

fn calibrate_wet(sensor: &Option<Arc<Mutex<Ads1115>>>, cfg: &mut AppConfig) {
    if let Some(sensor_arc) = sensor {
        if let Ok(mut sensor_lock) = sensor_arc.try_lock() {
            if let Ok(raw) = sensor_lock.read_moisture_sensor() {
                cfg.moisture_wet_value = Some(raw);
                let _ = save_config(cfg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_interval_bounds() {
        assert_eq!(clamp_interval(0), 50);
        assert_eq!(clamp_interval(49), 50);
        assert_eq!(clamp_interval(50), 50);
        assert_eq!(clamp_interval(100), 100);
        assert_eq!(clamp_interval(10_000), 10_000);
        assert_eq!(clamp_interval(20_000), 10_000);
    }
}

fn draw_ui(
    ctl: &Arc<dyn GpioController + Send + Sync>,
    pin: u8,
    invert: bool,
    sensor: &Option<Arc<Mutex<Ads1115>>>,
    cfg: &AppConfig,
) -> Result<(), Box<dyn Error>> {
    let mut out = stdout();

    execute!(out, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    let on = ctl.is_blink();
    let iv = ctl.interval_ms();

    let mut lines: Vec<String> = vec![
        "Plants Love Rust UI".to_string(),
        "------------------------------".to_string(),
        "".to_string(),
        format!("Pin {}: {}", pin, if on { "ON" } else { "OFF" }),
        format!("Interval: {} ms", iv),
        format!("Invert: {}", if invert { "ON" } else { "OFF" }),
        "".to_string(),
    ];

    // Moisture sensor readings
    if let Some(sensor_arc) = sensor {
        if let Ok(mut sensor_lock) = sensor_arc.try_lock() {
            match sensor_lock.read_moisture_sensor() {
                Ok(raw) => {
                    let voltage = Ads1115::raw_to_voltage(raw);
                    lines.push("Moisture Sensor (A3):".to_string());
                    lines.push(format!("  Raw: {}", raw));
                    lines.push(format!("  Voltage: {:.3}V", voltage));
                    
                    if let (Some(dry), Some(wet)) = (cfg.moisture_dry_value, cfg.moisture_wet_value) {
                        let pct = Ads1115::raw_to_moisture_percent(raw, dry, wet);
                        lines.push(format!("  Moisture: {:.1}%", pct));
                        lines.push(format!("  Cal: D={} W={}", dry, wet));
                    } else {
                        lines.push("  [Not calibrated]".to_string());
                    }
                }
                Err(e) => {
                    lines.push(format!("Sensor err: {}", e));
                }
            }
        } else {
            lines.push("Sensor: reading...".to_string());
        }
    } else {
        lines.push("Sensor: N/A".to_string());
    }

    lines.push("".to_string());
    lines.push("Controls:".to_string());
    lines.push("  q/Esc  - Quit".to_string());
    lines.push("  b      - Blink toggle".to_string());
    lines.push("  +/-    - Interval ms".to_string());
    lines.push("  d      - Cal dry".to_string());
    lines.push("  w      - Cal wet".to_string());

    draw_lines(&mut out, &lines)?;
    out.flush()?;
    Ok(())
}