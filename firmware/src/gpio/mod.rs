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

pub fn new_controller() -> Arc<dyn GpioController + Send + Sync> {
    #[cfg(feature = "gpio")]
    {
        Arc::new(RppalGpioController::new())
    }
    #[cfg(not(feature = "gpio"))]
    {
        Arc::new(NoopGpioController::new())
    }
}
