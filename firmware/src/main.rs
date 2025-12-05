// Entry point that wires together UI, config, and GPIO controller.

mod ui;
mod config;
mod gpio;

use crate::config::load_config;
use crate::gpio::new_controller;

fn main() {
    // Load persisted configuration
    let cfg = load_config();

    // Construct GPIO controller (real or stub depending on features)
    let controller = new_controller(cfg.gpio_pin);
    controller.set_blink(cfg.blink_on);
    controller.set_interval_ms(cfg.interval_ms);

    // Run the terminal UI only
    if let Err(e) = ui::run(controller, cfg) {
        eprintln!("TUI error: {e}");
    }
}