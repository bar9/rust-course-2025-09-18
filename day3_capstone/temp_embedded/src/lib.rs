#![no_std]

use heapless::{Vec, String};
use serde::{Deserialize, Serialize};

// Re-export core temperature types
pub use temp_core::Temperature;

// Fixed-capacity temperature reading for embedded systems
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedTemperatureReading {
    pub temperature: Temperature,
    pub timestamp: u32, // Using u32 for embedded systems (seconds since boot)
}

impl EmbeddedTemperatureReading {
    pub fn new(temperature: Temperature, timestamp: u32) -> Self {
        Self { temperature, timestamp }
    }
}

// Fixed-capacity storage for embedded systems
pub struct EmbeddedTemperatureStore<const N: usize> {
    readings: Vec<EmbeddedTemperatureReading, N>,
    total_readings: u32,
}

impl<const N: usize> EmbeddedTemperatureStore<N> {
    pub const fn new() -> Self {
        Self {
            readings: Vec::new(),
            total_readings: 0,
        }
    }

    pub fn add_reading(&mut self, reading: EmbeddedTemperatureReading) -> Result<(), &'static str> {
        self.total_readings += 1;

        if self.readings.len() >= N {
            // Circular buffer behavior - remove oldest reading
            self.readings.remove(0);
        }

        self.readings.push(reading).map_err(|_| "Storage full")?;
        Ok(())
    }

    pub fn get_latest(&self) -> Option<EmbeddedTemperatureReading> {
        self.readings.last().copied()
    }

    pub fn get_stats(&self) -> EmbeddedTemperatureStats {
        if self.readings.is_empty() {
            return EmbeddedTemperatureStats {
                min: Temperature::new(0.0),
                max: Temperature::new(0.0),
                average: Temperature::new(0.0),
                count: 0,
            };
        }

        let mut min_temp = self.readings[0].temperature.celsius;
        let mut max_temp = self.readings[0].temperature.celsius;
        let mut sum = 0.0;

        for reading in &self.readings {
            let temp = reading.temperature.celsius;
            if temp < min_temp {
                min_temp = temp;
            }
            if temp > max_temp {
                max_temp = temp;
            }
            sum += temp;
        }

        let average = sum / self.readings.len() as f32;

        EmbeddedTemperatureStats {
            min: Temperature::new(min_temp),
            max: Temperature::new(max_temp),
            average: Temperature::new(average),
            count: self.readings.len(),
        }
    }

    pub fn clear(&mut self) {
        self.readings.clear();
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub fn len(&self) -> usize {
        self.readings.len()
    }

    pub fn is_full(&self) -> bool {
        self.readings.len() >= N
    }

    pub fn is_empty(&self) -> bool {
        self.readings.is_empty()
    }

    pub fn total_readings(&self) -> u32 {
        self.total_readings
    }

    pub fn get_readings(&self) -> &[EmbeddedTemperatureReading] {
        &self.readings
    }
}

// Statistics without heap allocation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedTemperatureStats {
    pub min: Temperature,
    pub max: Temperature,
    pub average: Temperature,
    pub count: usize,
}

// Const configuration functions for zero-cost configuration
pub const fn calculate_sample_rate(desired_hz: u32, clock_hz: u32) -> u32 {
    clock_hz / desired_hz
}

pub const fn validate_buffer_size(size: usize) -> usize {
    assert!(size > 0 && size <= 1024, "Buffer size must be 1-1024");
    assert!(size & (size - 1) == 0, "Buffer size must be power of 2");
    size
}

pub const fn celsius_to_adc_value(celsius: f32) -> u16 {
    // Simple linear conversion: 10mV/°C, 3.3V reference, 12-bit ADC
    let voltage = celsius * 0.01; // 10mV/°C
    let adc_value = (voltage / 3.3) * 4095.0;
    adc_value as u16
}

// Configuration constants computed at compile time
pub const SYSTEM_CLOCK_HZ: u32 = 16_000_000; // 16 MHz
pub const SAMPLE_RATE_HZ: u32 = 10; // 10 Hz sampling
pub const TIMER_DIVISOR: u32 = calculate_sample_rate(SAMPLE_RATE_HZ, SYSTEM_CLOCK_HZ);
pub const READING_BUFFER_SIZE: usize = validate_buffer_size(64);
pub const TEMP_THRESHOLD_LOW: u16 = celsius_to_adc_value(5.0);   // 5°C
pub const TEMP_THRESHOLD_HIGH: u16 = celsius_to_adc_value(35.0); // 35°C
pub const TEMP_CRITICAL: u16 = celsius_to_adc_value(50.0);       // 50°C

// Binary protocol for embedded communication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EmbeddedCommand {
    GetStatus,
    GetLatestReading,
    GetReadingCount,
    GetStats,
    ClearReadings,
    SetSampleRate(u32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EmbeddedResponse {
    Status {
        uptime_seconds: u32,
        reading_count: u32,
        sample_rate: u32,
        buffer_usage: u8, // Percentage as u8 (0-100)
    },
    Reading(EmbeddedTemperatureReading),
    ReadingCount(u32),
    Stats(EmbeddedTemperatureStats),
    Cleared,
    SampleRateSet(u32),
    Error(u8), // Error code as u8 for compact binary encoding
}

pub struct EmbeddedProtocolHandler<const N: usize> {
    store: EmbeddedTemperatureStore<N>,
    sample_rate: u32,
    start_time: u32,
}

impl<const N: usize> EmbeddedProtocolHandler<N> {
    pub const fn new() -> Self {
        Self {
            store: EmbeddedTemperatureStore::new(),
            sample_rate: SAMPLE_RATE_HZ,
            start_time: 0,
        }
    }

    pub fn init(&mut self, start_time: u32) {
        self.start_time = start_time;
    }

    pub fn process_command(&mut self, command: EmbeddedCommand, current_time: u32) -> EmbeddedResponse {
        match command {
            EmbeddedCommand::GetStatus => {
                let uptime = current_time.saturating_sub(self.start_time);
                let buffer_usage = if N > 0 {
                    ((self.store.len() * 100) / N) as u8
                } else {
                    0
                };

                EmbeddedResponse::Status {
                    uptime_seconds: uptime,
                    reading_count: self.store.total_readings(),
                    sample_rate: self.sample_rate,
                    buffer_usage,
                }
            }
            EmbeddedCommand::GetLatestReading => {
                match self.store.get_latest() {
                    Some(reading) => EmbeddedResponse::Reading(reading),
                    None => EmbeddedResponse::Error(EmbeddedError::NoReadings.error_code()),
                }
            }
            EmbeddedCommand::GetReadingCount => {
                EmbeddedResponse::ReadingCount(self.store.total_readings())
            }
            EmbeddedCommand::GetStats => {
                EmbeddedResponse::Stats(self.store.get_stats())
            }
            EmbeddedCommand::ClearReadings => {
                self.store.clear();
                EmbeddedResponse::Cleared
            }
            EmbeddedCommand::SetSampleRate(rate) => {
                if rate > 0 && rate <= 1000 {
                    self.sample_rate = rate;
                    EmbeddedResponse::SampleRateSet(rate)
                } else {
                    EmbeddedResponse::Error(EmbeddedError::InvalidSampleRate.error_code())
                }
            }
        }
    }

    pub fn serialize_response(&self, response: &EmbeddedResponse) -> Result<Vec<u8, 256>, &'static str> {
        postcard::to_vec(response).map_err(|_| "Serialization failed")
    }

    pub fn deserialize_command(&self, data: &[u8]) -> Result<EmbeddedCommand, &'static str> {
        postcard::from_bytes(data).map_err(|_| "Deserialization failed")
    }

    pub fn add_reading(&mut self, temperature: Temperature, timestamp: u32) -> Result<(), &'static str> {
        let reading = EmbeddedTemperatureReading::new(temperature, timestamp);
        self.store.add_reading(reading)
    }

    pub fn get_store(&self) -> &EmbeddedTemperatureStore<N> {
        &self.store
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl<const N: usize> Default for EmbeddedProtocolHandler<N> {
    fn default() -> Self {
        Self::new()
    }
}

// Error types for embedded systems
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EmbeddedError {
    BufferFull,
    InvalidSampleRate,
    SensorTimeout,
    InvalidCommand,
    SerializationError,
    NoReadings,
}

impl EmbeddedError {
    pub const fn error_code(&self) -> u8 {
        match self {
            EmbeddedError::BufferFull => 1,
            EmbeddedError::InvalidSampleRate => 2,
            EmbeddedError::SensorTimeout => 3,
            EmbeddedError::InvalidCommand => 4,
            EmbeddedError::SerializationError => 5,
            EmbeddedError::NoReadings => 6,
        }
    }

    pub const fn description(&self) -> &'static str {
        match self {
            EmbeddedError::BufferFull => "Buffer full",
            EmbeddedError::InvalidSampleRate => "Invalid sample rate",
            EmbeddedError::SensorTimeout => "Sensor timeout",
            EmbeddedError::InvalidCommand => "Invalid command",
            EmbeddedError::SerializationError => "Serialization error",
            EmbeddedError::NoReadings => "No readings available",
        }
    }
}

// Utility function for creating fixed-capacity strings without std::format!
pub fn create_status_string(reading_count: u32, sample_rate: u32) -> String<128> {
    let mut status = String::new();
    status.push_str("Readings: ").ok();
    push_number(&mut status, reading_count as i32);
    status.push_str(", Rate: ").ok();
    push_number(&mut status, sample_rate as i32);
    status.push_str(" Hz").ok();
    status
}

pub fn format_temperature_reading(reading: &EmbeddedTemperatureReading) -> String<64> {
    let mut formatted = String::new();
    formatted.push_str("Temp: ").ok();
    push_float(&mut formatted, reading.temperature.celsius, 1);
    formatted.push_str("C @ ").ok();
    push_number(&mut formatted, reading.timestamp as i32);
    formatted.push('s').ok();
    formatted
}

fn push_number<const N: usize>(s: &mut String<N>, mut num: i32) {
    if num == 0 {
        s.push('0').ok();
        return;
    }

    if num < 0 {
        s.push('-').ok();
        num = -num;
    }

    let mut digits = Vec::<u8, 16>::new();
    while num > 0 {
        digits.push((num % 10) as u8).ok();
        num /= 10;
    }

    for &digit in digits.iter().rev() {
        s.push((b'0' + digit) as char).ok();
    }
}

fn push_float(s: &mut String<64>, mut value: f32, decimal_places: u8) {
    // Handle negative values
    if value < 0.0 {
        s.push('-').ok();
        value = -value;
    }

    // Extract integer part
    let integer_part = value as i32;
    push_number_small(s, integer_part);

    if decimal_places > 0 {
        s.push('.').ok();

        // Extract fractional part
        let mut fractional = value - integer_part as f32;
        for _ in 0..decimal_places {
            fractional *= 10.0;
            let digit = (fractional as i32) % 10;
            s.push((b'0' + digit as u8) as char).ok();
        }
    }
}

fn push_number_small(s: &mut String<64>, mut num: i32) {
    if num == 0 {
        s.push('0').ok();
        return;
    }

    let mut digits = Vec::<u8, 16>::new();
    while num > 0 {
        digits.push((num % 10) as u8).ok();
        num /= 10;
    }

    for &digit in digits.iter().rev() {
        s.push((b'0' + digit) as char).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_store_basic_operations() {
        let mut store: EmbeddedTemperatureStore<4> = EmbeddedTemperatureStore::new();

        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert_eq!(store.capacity(), 4);
        assert!(store.get_latest().is_none());

        // Add a reading
        let reading = EmbeddedTemperatureReading::new(Temperature::new(25.0), 1000);
        store.add_reading(reading).unwrap();

        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);
        assert_eq!(store.total_readings(), 1);

        let latest = store.get_latest().unwrap();
        assert_eq!(latest.temperature.celsius, 25.0);
        assert_eq!(latest.timestamp, 1000);
    }

    #[test]
    fn test_embedded_store_circular_buffer() {
        let mut store: EmbeddedTemperatureStore<3> = EmbeddedTemperatureStore::new();

        // Fill the buffer
        for i in 0..3 {
            let reading = EmbeddedTemperatureReading::new(Temperature::new(20.0 + i as f32), 1000 + i);
            store.add_reading(reading).unwrap();
        }

        assert_eq!(store.len(), 3);
        assert!(store.is_full());
        assert_eq!(store.total_readings(), 3);

        // Add one more - should trigger circular buffer behavior
        let reading = EmbeddedTemperatureReading::new(Temperature::new(25.0), 2000);
        store.add_reading(reading).unwrap();

        assert_eq!(store.len(), 3);
        assert_eq!(store.total_readings(), 4);

        // Should contain readings 21.0, 22.0, 25.0 (oldest removed)
        let readings = store.get_readings();
        assert_eq!(readings[0].temperature.celsius, 21.0);
        assert_eq!(readings[1].temperature.celsius, 22.0);
        assert_eq!(readings[2].temperature.celsius, 25.0);
    }

    #[test]
    fn test_embedded_store_statistics() {
        let mut store: EmbeddedTemperatureStore<5> = EmbeddedTemperatureStore::new();

        // Test empty store
        let stats = store.get_stats();
        assert_eq!(stats.count, 0);

        // Add some readings
        let temps = [10.0, 20.0, 30.0, 40.0, 50.0];
        for (i, &temp) in temps.iter().enumerate() {
            let reading = EmbeddedTemperatureReading::new(Temperature::new(temp), 1000 + i as u32);
            store.add_reading(reading).unwrap();
        }

        let stats = store.get_stats();
        assert_eq!(stats.min.celsius, 10.0);
        assert_eq!(stats.max.celsius, 50.0);
        assert_eq!(stats.average.celsius, 30.0);
        assert_eq!(stats.count, 5);
    }

    #[test]
    fn test_const_configuration() {
        // Test compile-time constants
        assert_eq!(SYSTEM_CLOCK_HZ, 16_000_000);
        assert_eq!(SAMPLE_RATE_HZ, 10);
        assert_eq!(TIMER_DIVISOR, 1_600_000);
        assert_eq!(READING_BUFFER_SIZE, 64);

        // Test const functions
        assert_eq!(calculate_sample_rate(100, 16_000_000), 160_000);
        assert_eq!(validate_buffer_size(32), 32);

        // Test temperature thresholds
        assert!(TEMP_THRESHOLD_LOW < TEMP_THRESHOLD_HIGH);
        assert!(TEMP_THRESHOLD_HIGH < TEMP_CRITICAL);
    }

    #[test]
    fn test_protocol_handler() {
        let mut handler: EmbeddedProtocolHandler<8> = EmbeddedProtocolHandler::new();
        handler.init(1000);

        // Test GetStatus command
        let response = handler.process_command(EmbeddedCommand::GetStatus, 2000);
        if let EmbeddedResponse::Status { uptime_seconds, reading_count, sample_rate, buffer_usage } = response {
            assert_eq!(uptime_seconds, 1000);
            assert_eq!(reading_count, 0);
            assert_eq!(sample_rate, SAMPLE_RATE_HZ);
            assert_eq!(buffer_usage, 0);
        } else {
            panic!("Expected Status response");
        }

        // Add a reading and test again
        handler.add_reading(Temperature::new(23.5), 1500).unwrap();

        let response = handler.process_command(EmbeddedCommand::GetLatestReading, 2000);
        if let EmbeddedResponse::Reading(reading) = response {
            assert_eq!(reading.temperature.celsius, 23.5);
            assert_eq!(reading.timestamp, 1500);
        } else {
            panic!("Expected Reading response");
        }

        // Test reading count
        let response = handler.process_command(EmbeddedCommand::GetReadingCount, 2000);
        if let EmbeddedResponse::ReadingCount(count) = response {
            assert_eq!(count, 1);
        } else {
            panic!("Expected ReadingCount response");
        }

        // Test sample rate setting
        let response = handler.process_command(EmbeddedCommand::SetSampleRate(20), 2000);
        if let EmbeddedResponse::SampleRateSet(rate) = response {
            assert_eq!(rate, 20);
            assert_eq!(handler.get_sample_rate(), 20);
        } else {
            panic!("Expected SampleRateSet response");
        }
    }

    #[test]
    fn test_protocol_serde_serialization() {
        let handler: EmbeddedProtocolHandler<8> = EmbeddedProtocolHandler::new();

        // Test command serialization/deserialization
        let command = EmbeddedCommand::GetStatus;
        let serialized_command = postcard::to_vec::<_, 64>(&command).unwrap();
        let deserialized_command = handler.deserialize_command(&serialized_command).unwrap();
        assert_eq!(deserialized_command, EmbeddedCommand::GetStatus);

        // Test response serialization
        let response = EmbeddedResponse::Status {
            uptime_seconds: 1000,
            reading_count: 42,
            sample_rate: 10,
            buffer_usage: 50,
        };

        let serialized = handler.serialize_response(&response).unwrap();
        // Postcard produces compact binary output
        assert!(serialized.len() > 0 && serialized.len() < 32);

        // Test command with parameter
        let command_with_param = EmbeddedCommand::SetSampleRate(100);
        let serialized_command = postcard::to_vec::<_, 64>(&command_with_param).unwrap();
        let deserialized_command = handler.deserialize_command(&serialized_command).unwrap();
        assert_eq!(deserialized_command, EmbeddedCommand::SetSampleRate(100));
    }

    #[test]
    fn test_error_handling() {
        let mut handler: EmbeddedProtocolHandler<2> = EmbeddedProtocolHandler::new();

        // Test no readings error
        let response = handler.process_command(EmbeddedCommand::GetLatestReading, 1000);
        if let EmbeddedResponse::Error(code) = response {
            assert_eq!(code, EmbeddedError::NoReadings.error_code());
        } else {
            panic!("Expected error response");
        }

        // Test invalid sample rate
        let response = handler.process_command(EmbeddedCommand::SetSampleRate(0), 1000);
        if let EmbeddedResponse::Error(code) = response {
            assert_eq!(code, EmbeddedError::InvalidSampleRate.error_code());
        } else {
            panic!("Expected error response");
        }

        let response = handler.process_command(EmbeddedCommand::SetSampleRate(2000), 1000);
        if let EmbeddedResponse::Error(code) = response {
            assert_eq!(code, EmbeddedError::InvalidSampleRate.error_code());
        } else {
            panic!("Expected error response");
        }
    }

    #[test]
    fn test_string_formatting() {
        let status = create_status_string(42, 10);
        assert_eq!(status.as_str(), "Readings: 42, Rate: 10 Hz");

        let reading = EmbeddedTemperatureReading::new(Temperature::new(23.5), 1500);
        let formatted = format_temperature_reading(&reading);
        assert_eq!(formatted.as_str(), "Temp: 23.5C @ 1500s");
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(EmbeddedError::BufferFull.error_code(), 1);
        assert_eq!(EmbeddedError::InvalidSampleRate.error_code(), 2);
        assert_eq!(EmbeddedError::SensorTimeout.error_code(), 3);
        assert_eq!(EmbeddedError::InvalidCommand.error_code(), 4);
        assert_eq!(EmbeddedError::SerializationError.error_code(), 5);
        assert_eq!(EmbeddedError::NoReadings.error_code(), 6);

        assert_eq!(EmbeddedError::BufferFull.description(), "Buffer full");
        assert_eq!(EmbeddedError::NoReadings.description(), "No readings available");
    }
}