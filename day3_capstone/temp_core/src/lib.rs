#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Temperature {
    pub celsius: f32,
}

impl Temperature {
    pub fn new(celsius: f32) -> Self {
        Self { celsius }
    }

    pub fn from_fahrenheit(fahrenheit: f32) -> Self {
        Self {
            celsius: (fahrenheit - 32.0) * 5.0 / 9.0,
        }
    }

    pub fn from_kelvin(kelvin: f32) -> Self {
        Self {
            celsius: kelvin - 273.15,
        }
    }

    /// Convert from embedded sensor ADC value to temperature
    /// Assumes 10mV/°C sensor with 3.3V reference and 12-bit ADC
    pub fn from_embedded_sensor(adc_value: u16) -> Self {
        let voltage = (adc_value as f32 / 4095.0) * 3.3;
        let celsius = voltage / 0.01; // 10mV/°C sensor
        Self { celsius }
    }

    pub fn to_fahrenheit(&self) -> f32 {
        self.celsius * 9.0 / 5.0 + 32.0
    }

    pub fn to_kelvin(&self) -> f32 {
        self.celsius + 273.15
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}°C", self.celsius)
    }
}

pub trait TemperatureSensor {
    type Error: fmt::Debug;

    fn read_temperature(&mut self) -> Result<Temperature, Self::Error>;
    fn sensor_id(&self) -> &str;
}

#[cfg(feature = "std")]
pub mod mock;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature_conversions() {
        let temp = Temperature::new(20.0);
        assert!((temp.to_fahrenheit() - 68.0).abs() < 0.1);
        assert!((temp.to_kelvin() - 293.15).abs() < 0.1);

        let from_f = Temperature::from_fahrenheit(68.0);
        assert!((from_f.celsius - 20.0).abs() < 0.1);

        let from_k = Temperature::from_kelvin(293.15);
        assert!((from_k.celsius - 20.0).abs() < 0.1);
    }

    #[test]
    fn temperature_display() {
        let temp = Temperature::new(23.456);
        assert_eq!(format!("{}", temp), "23.5°C");
    }
}
