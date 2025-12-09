#[cfg(feature = "gpio")]
use rppal::i2c::I2c;
#[cfg(feature = "gpio")]
use std::thread;
#[cfg(feature = "gpio")]
use std::time::Duration;

/// ADS1115 16-bit ADC I2C address
#[cfg(feature = "gpio")]
const ADS1115_ADDRESS: u16 = 0x48;

/// ADS1115 Register addresses
#[cfg(feature = "gpio")]
const REG_CONVERSION: u8 = 0x00;
#[cfg(feature = "gpio")]
const REG_CONFIG: u8 = 0x01;

/// Input multiplexer configuration (AINp = A3, AINn = GND)
#[cfg(feature = "gpio")]
const MUX_A3_GND: u16 = 0b111 << 12;

/// Programmable gain amplifier (±4.096V range)
#[cfg(feature = "gpio")]
const PGA_4_096V: u16 = 0b001 << 9;

/// Operating mode (single-shot)
#[cfg(feature = "gpio")]
const MODE_SINGLE: u16 = 0b1 << 8;

/// Data rate (128 SPS)
#[cfg(feature = "gpio")]
const DATA_RATE_128SPS: u16 = 0b100 << 5;

/// Comparator mode (traditional)
#[cfg(feature = "gpio")]
const COMP_MODE_TRAD: u16 = 0b0 << 4;

/// Comparator polarity (active low)
#[cfg(feature = "gpio")]
const COMP_POL_LOW: u16 = 0b0 << 3;

/// Latching comparator (non-latching)
#[cfg(feature = "gpio")]
const COMP_LAT_NON: u16 = 0b0 << 2;

/// Comparator queue (disable)
#[cfg(feature = "gpio")]
const COMP_QUE_DIS: u16 = 0b11;

/// Operational status (start single conversion)
#[cfg(feature = "gpio")]
const OS_START_SINGLE: u16 = 0b1 << 15;

/// Operational status (conversion ready mask)
#[cfg(feature = "gpio")]
const OS_READY: u16 = 0b1 << 15;

pub struct Ads1115 {
    #[cfg(feature = "gpio")]
    i2c: I2c,
    #[cfg(not(feature = "gpio"))]
    _phantom: (),
}

impl Ads1115 {
    /// Create a new ADS1115 instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(feature = "gpio")]
        {
            let mut i2c = I2c::new()?;
            i2c.set_slave_address(ADS1115_ADDRESS)?;
            Ok(Self { i2c })
        }
        #[cfg(not(feature = "gpio"))]
        {
            Err("ADS1115 requires 'gpio' feature to be enabled".into())
        }
    }

    /// Read the moisture sensor value from A3 input
    /// Returns a raw ADC value (0-32767 for positive values)
    pub fn read_moisture_sensor(&mut self) -> Result<i16, Box<dyn std::error::Error>> {
        #[cfg(feature = "gpio")]
        {
            self.read_channel_a3()
        }
        #[cfg(not(feature = "gpio"))]
        {
            Err("GPIO feature not enabled".into())
        }
    }

    /// Read from A3 channel (single-ended measurement against GND)
    #[cfg(feature = "gpio")]
    pub fn read_channel_a3(&mut self) -> Result<i16, Box<dyn std::error::Error>> {
        // Configure ADC for A3 single-ended measurement
        let config = OS_START_SINGLE
            | MUX_A3_GND
            | PGA_4_096V
            | MODE_SINGLE
            | DATA_RATE_128SPS
            | COMP_MODE_TRAD
            | COMP_POL_LOW
            | COMP_LAT_NON
            | COMP_QUE_DIS;

        // Write configuration register
        self.write_register(REG_CONFIG, config)?;

        // Wait for conversion to complete (at 128 SPS, ~8ms)
        thread::sleep(Duration::from_millis(10));

        // Wait for conversion ready
        let mut attempts = 0;
        loop {
            let config_read = self.read_register(REG_CONFIG)?;
            if (config_read & OS_READY) != 0 {
                break;
            }
            thread::sleep(Duration::from_millis(1));
            attempts += 1;
            if attempts > 100 {
                return Err("Timeout waiting for conversion".into());
            }
        }

        // Read conversion result
        let raw = self.read_register(REG_CONVERSION)?;
        Ok(raw as i16)
    }

    /// Convert raw ADC value to voltage (±4.096V range)
    pub fn raw_to_voltage(raw: i16) -> f32 {
        // For ±4.096V range: LSB = 0.125 mV
        (raw as f32) * 0.000125
    }

    /// Convert raw ADC value to moisture percentage estimate
    /// Note: Calibration values will need to be adjusted based on your specific sensor
    /// Typical ranges: ~13000-15000 (dry in air) to ~26000-28000 (in water)
    pub fn raw_to_moisture_percent(raw: i16, dry_value: i16, wet_value: i16) -> f32 {
        if wet_value == dry_value {
            return 0.0;
        }
        
        let percentage = ((raw - dry_value) as f32 / (wet_value - dry_value) as f32) * 100.0;
        percentage.clamp(0.0, 100.0)
    }

    /// Write to a 16-bit register
    #[cfg(feature = "gpio")]
    fn write_register(&mut self, register: u8, value: u16) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = [
            register,
            (value >> 8) as u8,  // MSB
            (value & 0xFF) as u8, // LSB
        ];
        self.i2c.write(&bytes)?;
        Ok(())
    }

    /// Read from a 16-bit register
    #[cfg(feature = "gpio")]
    fn read_register(&mut self, register: u8) -> Result<u16, Box<dyn std::error::Error>> {
        self.i2c.write(&[register])?;
        let mut buffer = [0u8; 2];
        self.i2c.read(&mut buffer)?;
        Ok(((buffer[0] as u16) << 8) | (buffer[1] as u16))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_to_voltage() {
        assert_eq!(Ads1115::raw_to_voltage(0), 0.0);
        assert!((Ads1115::raw_to_voltage(32767) - 4.096).abs() < 0.001);
        assert!((Ads1115::raw_to_voltage(-32768) - (-4.096)).abs() < 0.001);
    }

    #[test]
    fn test_raw_to_moisture_percent() {
        let dry = 15000;
        let wet = 27000;
        
        assert_eq!(Ads1115::raw_to_moisture_percent(dry, dry, wet), 0.0);
        assert_eq!(Ads1115::raw_to_moisture_percent(wet, dry, wet), 100.0);
        assert!((Ads1115::raw_to_moisture_percent(21000, dry, wet) - 50.0).abs() < 0.1);
    }

    #[test]
    #[ignore]
    fn test_read_sensor() {
        // This test requires actual hardware
        let mut ads = Ads1115::new().expect("Failed to create ADS1115");
        let raw = ads.read_moisture_sensor().expect("Failed to read sensor");
        println!("Raw value: {}", raw);
        println!("Voltage: {:.3}V", Ads1115::raw_to_voltage(raw));
    }
}
