// Minimal firmware scaffold for the PiGrow / plants-love-rust project.
// Replace this with real firmware logic (GPIO, sensors, actuators, etc.).

#[cfg(feature = "gpio")]
mod gpio_example;

fn main() {
    println!("Plants Love Rust — firmware scaffold (hello world)");

    #[cfg(feature = "gpio")]
    {
        if let Err(e) = gpio_example::run_gpio_example() {
            eprintln!("GPIO example error: {}", e);
        }
    }
}
