// GPIO example for Raspberry Pi using the `rppal` crate.
// This is compiled only when the Cargo feature `gpio` is enabled.

#[cfg(feature = "gpio")]
pub fn run_gpio_example() -> Result<(), Box<dyn std::error::Error>> {
    use rppal::gpio::Gpio;
    use std::thread::sleep;
    use std::time::Duration;

    const PIN: u8 = 17; // physical BCM pin 17 (GPIO17) — change as needed

    let gpio = Gpio::new()?;
    let mut pin = gpio.get(PIN)?.into_output();

    // Blink the pin a few times
    while true {
        pin.set_high();
        sleep(Duration::from_millis(500));
        pin.set_low();
        sleep(Duration::from_millis(500));
    }

    Ok(())
}
