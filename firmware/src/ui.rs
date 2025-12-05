use crate::config::{save_config, AppConfig};
use crate::gpio::GpioController;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal;
use crossterm::{cursor, execute, style, terminal::ClearType};
use std::error::Error;
use std::io::{stdout, Write};
use std::sync::Arc;
use std::time::Duration;

pub fn run(
    ctl: Arc<dyn GpioController + Send + Sync>,
    mut cfg: AppConfig,
) -> Result<(), Box<dyn Error>> {
    let mut out = stdout();
    terminal::enable_raw_mode()?;
    execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut running = true;
    loop {
        draw_ui(&ctl)?;

        if event::poll(Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(k) => match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => running = false,
                    KeyCode::Char('b') => {
                        let new = !ctl.is_blink();
                        ctl.set_blink(new);
                        cfg.blink_on = new;
                        let _ = save_config(&cfg);
                    }
                    KeyCode::Char('+') => {
                        let cur = ctl.interval_ms();
                        let next = cur.saturating_sub(100).max(50);
                        ctl.set_interval_ms(next);
                        cfg.interval_ms = next;
                        let _ = save_config(&cfg);
                    }
                    KeyCode::Char('-') => {
                        let cur = ctl.interval_ms();
                        let next = cur.saturating_add(100).min(10_000);
                        ctl.set_interval_ms(next);
                        cfg.interval_ms = next;
                        let _ = save_config(&cfg);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if !running { break; }
    }

    execute!(out, terminal::LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn draw_ui(ctl: &Arc<dyn GpioController + Send + Sync>) -> Result<(), Box<dyn Error>> {
    let mut out = stdout();

    execute!(
        out,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        style::Print("Plants Love Rust — Basic UI\n"),
        style::Print("--------------------------------\n\n"),
    )?;

    let on = ctl.is_blink();
    let iv = ctl.interval_ms();

    execute!(
        out,
        style::Print(format!("GPIO17 Blink: {}\n", if on { "ON" } else { "OFF" })),
        style::Print(format!("Interval: {} ms\n\n", iv)),
    )?;

    execute!(
        out,
        style::Print("Controls:\n"),
        style::Print("  q / Esc  - Quit\n"),
        style::Print("  b        - Toggle GPIO17 blink\n"),
        style::Print("  + / -    - Adjust blink interval\n\n"),
        style::Print("Tip: On the Pi GUI, run in a terminal window for interactive control.\n"),
    )?;

    let (cols, rows) = terminal::size()?;
    let status = format!(
        "[q:Quit] [b:Blink {}] [+/-:Interval {} ms]  {}x{}",
        if on { "ON" } else { "OFF" },
        iv,
        cols,
        rows
    );

    execute!(
        out,
        cursor::MoveTo(0, rows.saturating_sub(1)),
        terminal::Clear(ClearType::CurrentLine),
        style::Print(status),
    )?;

    out.flush()?;
    Ok(())
}
