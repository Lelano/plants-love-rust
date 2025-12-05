use std::sync::Arc;

pub trait GpioController: Send + Sync {
    fn set_blink(&self, on: bool);
    fn is_blink(&self) -> bool;
    fn set_interval_ms(&self, ms: u64);
    fn interval_ms(&self) -> u64;
}

#[cfg(feature = "gpio")]
mod real;
#[cfg(not(feature = "gpio"))]
mod stub;

#[cfg(feature = "gpio")]
pub use real::RppalGpioController;
#[cfg(not(feature = "gpio"))]
pub use stub::NoopGpioController;

pub fn new_controller(_gpio_pin: u8) -> Arc<dyn GpioController + Send + Sync> {
    #[cfg(feature = "gpio")]
    {
        Arc::new(RppalGpioController::new(_gpio_pin))
    }
    #[cfg(not(feature = "gpio"))]
    {
        Arc::new(NoopGpioController::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controller_roundtrip() {
        let ctl = new_controller(17);
        ctl.set_blink(true);
        assert!(ctl.is_blink());
        ctl.set_blink(false);
        assert!(!ctl.is_blink());

        ctl.set_interval_ms(777);
        assert_eq!(ctl.interval_ms(), 777);
    }
}
