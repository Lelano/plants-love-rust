// GPIO example for Raspberry Pi using the `rppal` crate.
// This is compiled only when the Cargo feature `gpio` is enabled.

#[cfg(feature = "gpio")]
pub fn run_gpio_example() -> Result<(), Box<dyn std::error::Error>> {
    use rppal::gpio::Gpio;
    use std::thread::sleep;
    use std::time::Duration;
    use std::io::{self, Write};

    const PIN: u8 = 17; // physical BCM pin 17 (GPIO17) — change as needed

    let gpio = Gpio::new()?;
    let mut pin = gpio.get(PIN)?.into_output();

    // Blink the pin continuously and log transitions
    loop {
        pin.set_high();
        println!("GPIO{} -> HIGH", PIN);
        io::stdout().flush().ok();
        sleep(Duration::from_millis(1000));

        pin.set_low();
        println!("GPIO{} -> LOW", PIN);
        io::stdout().flush().ok();
        sleep(Duration::from_millis(1000));
    }
}
