use crate::{Temperature, TemperatureSensor};
use std::fmt;
extern crate alloc;
use alloc::string::String;

#[derive(Debug)]
pub enum MockError {
    SensorOffline,
    ReadFailed,
}

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockError::SensorOffline => write!(f, "Sensor is offline"),
            MockError::ReadFailed => write!(f, "Failed to read sensor"),
        }
    }
}

pub struct MockTemperatureSensor {
    id: String,
    temperature: f32,
    fail_next: bool,
    offline: bool,
}

impl MockTemperatureSensor {
    pub fn new(id: String, temperature: f32) -> Self {
        Self {
            id,
            temperature,
            fail_next: false,
            offline: false,
        }
    }

    pub fn set_temperature(&mut self, temp: f32) {
        self.temperature = temp;
    }

    pub fn set_base_temperature(&mut self, temp: f32) {
        self.temperature = temp;
    }

    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }

    pub fn fail_next_read(&mut self) {
        self.fail_next = true;
    }
}

impl TemperatureSensor for MockTemperatureSensor {
    type Error = MockError;

    fn read_temperature(&mut self) -> Result<Temperature, Self::Error> {
        if self.offline {
            return Err(MockError::SensorOffline);
        }

        if self.fail_next {
            self.fail_next = false;
            return Err(MockError::ReadFailed);
        }

        Ok(Temperature::new(self.temperature))
    }

    fn sensor_id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_sensor_works() {
        let mut sensor = MockTemperatureSensor::new("test-sensor".to_string(), 25.0);

        let reading = sensor.read_temperature().unwrap();
        assert_eq!(reading.celsius, 25.0);
        assert_eq!(sensor.sensor_id(), "test-sensor");
    }

    #[test]
    fn mock_sensor_can_fail() {
        let mut sensor = MockTemperatureSensor::new("test-sensor".to_string(), 25.0);

        sensor.fail_next_read();
        let result = sensor.read_temperature();
        assert!(matches!(result, Err(MockError::ReadFailed)));

        // Should work again after failure
        let reading = sensor.read_temperature().unwrap();
        assert_eq!(reading.celsius, 25.0);
    }

    #[test]
    fn mock_sensor_can_be_offline() {
        let mut sensor = MockTemperatureSensor::new("test-sensor".to_string(), 25.0);

        sensor.set_offline(true);
        let result = sensor.read_temperature();
        assert!(matches!(result, Err(MockError::SensorOffline)));

        sensor.set_offline(false);
        let reading = sensor.read_temperature().unwrap();
        assert_eq!(reading.celsius, 25.0);
    }

    #[test]
    fn mock_sensor_temperature_can_change() {
        let mut sensor = MockTemperatureSensor::new("test-sensor".to_string(), 25.0);

        let reading1 = sensor.read_temperature().unwrap();
        assert_eq!(reading1.celsius, 25.0);

        sensor.set_temperature(30.0);
        let reading2 = sensor.read_temperature().unwrap();
        assert_eq!(reading2.celsius, 30.0);
    }
}