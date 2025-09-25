use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use temp_core::{TemperatureSensor, mock::MockTemperatureSensor};
use temp_store::{TemperatureStore, TemperatureStats, TemperatureReading};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Command {
    GetStatus,
    GetReading {
        sensor_id: String
    },
    SetThreshold {
        sensor_id: String,
        min_temp: f32,
        max_temp: f32,
    },
    GetHistory {
        sensor_id: String,
        last_n: usize,
    },
    GetStats {
        sensor_id: String,
    },
    Calibrate {
        sensor_id: String,
        actual_temp: f32,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Response {
    Status {
        active_sensors: Vec<String>,
        uptime_seconds: u64,
        readings_count: usize,
    },
    Reading {
        sensor_id: String,
        temperature: f32,
        timestamp: u64,
    },
    ThresholdSet {
        sensor_id: String,
        min_temp: f32,
        max_temp: f32,
    },
    History {
        sensor_id: String,
        readings: Vec<TemperatureReading>,
    },
    Stats {
        sensor_id: String,
        stats: TemperatureStats,
    },
    CalibrationComplete {
        sensor_id: String,
        offset_adjustment: f32,
    },
    Error {
        code: u16,
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProtocolMessage {
    pub version: u8,
    pub id: u32,
    pub payload: MessagePayload,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MessagePayload {
    Command(Command),
    Response(Response),
}

#[derive(Debug, Clone)]
pub enum ProtocolError {
    InvalidSensorId { sensor_id: String },
    SensorNotResponding { sensor_id: String },
    InvalidThreshold { min: f32, max: f32, reason: String },
    CalibrationFailed { sensor_id: String, reason: String },
    SystemError { code: u16, details: String },
    ProtocolVersionMismatch { expected: u8, received: u8 },
}

impl ProtocolError {
    pub fn to_response(&self) -> Response {
        match self {
            ProtocolError::InvalidSensorId { sensor_id } => Response::Error {
                code: 404,
                message: format!("Sensor '{}' not found", sensor_id),
            },
            ProtocolError::SensorNotResponding { sensor_id } => Response::Error {
                code: 503,
                message: format!("Sensor '{}' is not responding", sensor_id),
            },
            ProtocolError::InvalidThreshold { min, max, reason } => Response::Error {
                code: 400,
                message: format!("Invalid threshold min={}, max={}: {}", min, max, reason),
            },
            ProtocolError::CalibrationFailed { sensor_id, reason } => Response::Error {
                code: 422,
                message: format!("Calibration failed for '{}': {}", sensor_id, reason),
            },
            ProtocolError::SystemError { code, details } => Response::Error {
                code: *code,
                message: details.clone(),
            },
            ProtocolError::ProtocolVersionMismatch { expected, received } => Response::Error {
                code: 505,
                message: format!("Protocol version mismatch: expected {}, got {}", expected, received),
            },
        }
    }
}

pub struct TemperatureProtocolHandler {
    next_message_id: u32,
    sensors: HashMap<String, MockTemperatureSensor>,
    store: TemperatureStore,
    thresholds: HashMap<String, (f32, f32)>,
    start_time: std::time::Instant,
}

impl TemperatureProtocolHandler {
    pub fn new() -> Self {
        let mut sensors = HashMap::new();

        // Initialize with some mock sensors
        sensors.insert("temp_01".to_string(),
                      MockTemperatureSensor::new("temp_01".to_string(), 23.5));
        sensors.insert("temp_02".to_string(),
                      MockTemperatureSensor::new("temp_02".to_string(), 21.8));
        sensors.insert("temp_03".to_string(),
                      MockTemperatureSensor::new("temp_03".to_string(), 25.1));

        Self {
            next_message_id: 1,
            sensors,
            store: TemperatureStore::new(100), // Capacity of 100 readings
            thresholds: HashMap::new(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn create_command(&mut self, command: Command) -> ProtocolMessage {
        let id = self.next_message_id;
        self.next_message_id += 1;

        ProtocolMessage {
            version: 1,
            id,
            payload: MessagePayload::Command(command),
        }
    }

    pub fn create_response(&self, request_id: u32, response: Response) -> ProtocolMessage {
        ProtocolMessage {
            version: 1,
            id: request_id,
            payload: MessagePayload::Response(response),
        }
    }

    pub fn process_command(&mut self, message: ProtocolMessage) -> ProtocolMessage {
        // Check protocol version
        if message.version != 1 {
            let error = ProtocolError::ProtocolVersionMismatch {
                expected: 1,
                received: message.version
            };
            return self.create_response(message.id, error.to_response());
        }

        let response = match message.payload {
            MessagePayload::Command(command) => self.handle_command(command),
            MessagePayload::Response(_) => {
                Response::Error {
                    code: 400,
                    message: "Cannot process response messages".to_string(),
                }
            }
        };

        self.create_response(message.id, response)
    }

    fn handle_command(&mut self, command: Command) -> Response {
        match command {
            Command::GetStatus => {
                let active_sensors: Vec<String> = self.sensors.keys().cloned().collect();
                Response::Status {
                    active_sensors,
                    uptime_seconds: self.start_time.elapsed().as_secs(),
                    readings_count: self.store.reading_count(),
                }
            }
            Command::GetReading { sensor_id } => {
                if let Some(sensor) = self.sensors.get_mut(&sensor_id) {
                    match sensor.read_temperature() {
                        Ok(temp) => {
                            let reading = TemperatureReading::new(temp);
                            self.store.add_reading(reading);

                            Response::Reading {
                                sensor_id,
                                temperature: temp.celsius,
                                timestamp: reading.timestamp,
                            }
                        }
                        Err(_) => {
                            let error = ProtocolError::SensorNotResponding { sensor_id };
                            error.to_response()
                        }
                    }
                } else {
                    let error = ProtocolError::InvalidSensorId { sensor_id };
                    error.to_response()
                }
            }
            Command::SetThreshold { sensor_id, min_temp, max_temp } => {
                if min_temp >= max_temp {
                    let error = ProtocolError::InvalidThreshold {
                        min: min_temp,
                        max: max_temp,
                        reason: "Min temperature must be less than max temperature".to_string(),
                    };
                    return error.to_response();
                }

                if !self.sensors.contains_key(&sensor_id) {
                    let error = ProtocolError::InvalidSensorId { sensor_id };
                    return error.to_response();
                }

                self.thresholds.insert(sensor_id.clone(), (min_temp, max_temp));
                Response::ThresholdSet {
                    sensor_id,
                    min_temp,
                    max_temp,
                }
            }
            Command::GetHistory { sensor_id, last_n } => {
                if !self.sensors.contains_key(&sensor_id) {
                    let error = ProtocolError::InvalidSensorId { sensor_id };
                    return error.to_response();
                }

                let readings = self.store.get_recent_readings(last_n);
                Response::History {
                    sensor_id,
                    readings,
                }
            }
            Command::GetStats { sensor_id } => {
                if !self.sensors.contains_key(&sensor_id) {
                    let error = ProtocolError::InvalidSensorId { sensor_id };
                    return error.to_response();
                }

                let stats = self.store.get_stats();
                Response::Stats {
                    sensor_id,
                    stats,
                }
            }
            Command::Calibrate { sensor_id, actual_temp } => {
                if let Some(sensor) = self.sensors.get_mut(&sensor_id) {
                    // Simulate calibration by reading current temperature and calculating offset
                    match sensor.read_temperature() {
                        Ok(current_temp) => {
                            let offset = actual_temp - current_temp.celsius;
                            sensor.set_base_temperature(actual_temp);

                            Response::CalibrationComplete {
                                sensor_id,
                                offset_adjustment: offset,
                            }
                        }
                        Err(_) => {
                            let error = ProtocolError::CalibrationFailed {
                                sensor_id,
                                reason: "Sensor not responding during calibration".to_string(),
                            };
                            error.to_response()
                        }
                    }
                } else {
                    let error = ProtocolError::InvalidSensorId { sensor_id };
                    error.to_response()
                }
            }
        }
    }

    pub fn serialize_json(&self, message: &ProtocolMessage) -> Result<String, serde_json::Error> {
        serde_json::to_string(message)
    }

    pub fn serialize_binary(&self, message: &ProtocolMessage) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(message)
    }

    pub fn deserialize_json(&self, data: &str) -> Result<ProtocolMessage, serde_json::Error> {
        serde_json::from_str(data)
    }

    pub fn deserialize_binary(&self, data: &[u8]) -> Result<ProtocolMessage, postcard::Error> {
        postcard::from_bytes(data)
    }
}

impl Default for TemperatureProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization() {
        let command = Command::GetReading {
            sensor_id: "temp_01".to_string(),
        };

        let message = ProtocolMessage {
            version: 1,
            id: 123,
            payload: MessagePayload::Command(command),
        };

        // Test JSON serialization
        let json_str = serde_json::to_string(&message).unwrap();
        let parsed_message: ProtocolMessage = serde_json::from_str(&json_str).unwrap();
        assert_eq!(message, parsed_message);

        // Test binary serialization
        let binary_data = postcard::to_allocvec(&message).unwrap();
        let parsed_message: ProtocolMessage = postcard::from_bytes(&binary_data).unwrap();
        assert_eq!(message, parsed_message);
    }

    #[test]
    fn test_binary_vs_json_size() {
        let command = Command::GetHistory {
            sensor_id: "temp_sensor_with_very_long_name_for_testing".to_string(),
            last_n: 100,
        };

        let message = ProtocolMessage {
            version: 1,
            id: 12345,
            payload: MessagePayload::Command(command),
        };

        let json_data = serde_json::to_string(&message).unwrap();
        let binary_data = postcard::to_allocvec(&message).unwrap();

        println!("JSON size: {} bytes", json_data.len());
        println!("Binary size: {} bytes", binary_data.len());

        // Binary should be significantly smaller than JSON
        assert!(binary_data.len() < json_data.len());

        // For this message, we expect at least 30% space savings
        let savings_ratio = (json_data.len() - binary_data.len()) as f32 / json_data.len() as f32;
        assert!(savings_ratio > 0.3, "Expected at least 30% space savings, got {:.1}%", savings_ratio * 100.0);
    }

    #[test]
    fn test_protocol_versioning() {
        let mut handler = TemperatureProtocolHandler::new();

        // Create message with wrong version
        let message = ProtocolMessage {
            version: 2, // Wrong version
            id: 1,
            payload: MessagePayload::Command(Command::GetStatus),
        };

        let response = handler.process_command(message);

        if let MessagePayload::Response(Response::Error { code, message: msg }) = response.payload {
            assert_eq!(code, 505);
            assert!(msg.contains("version mismatch"));
        } else {
            panic!("Expected version mismatch error");
        }
    }

    #[test]
    fn test_error_responses() {
        let mut handler = TemperatureProtocolHandler::new();

        // Test invalid sensor ID
        let message = handler.create_command(Command::GetReading {
            sensor_id: "nonexistent_sensor".to_string(),
        });

        let response = handler.process_command(message);

        if let MessagePayload::Response(Response::Error { code, message: msg }) = response.payload {
            assert_eq!(code, 404);
            assert!(msg.contains("not found"));
        } else {
            panic!("Expected sensor not found error");
        }

        // Test invalid threshold
        let message = handler.create_command(Command::SetThreshold {
            sensor_id: "temp_01".to_string(),
            min_temp: 30.0,
            max_temp: 20.0, // Invalid: min > max
        });

        let response = handler.process_command(message);

        if let MessagePayload::Response(Response::Error { code, message: msg }) = response.payload {
            assert_eq!(code, 400);
            assert!(msg.contains("Invalid threshold"));
        } else {
            panic!("Expected invalid threshold error");
        }
    }

    #[test]
    fn test_command_processing() {
        let mut handler = TemperatureProtocolHandler::new();

        // Test GetStatus command
        let message = handler.create_command(Command::GetStatus);
        let response = handler.process_command(message);

        if let MessagePayload::Response(Response::Status { active_sensors, uptime_seconds: _, readings_count }) = response.payload {
            assert_eq!(active_sensors.len(), 3); // We have 3 mock sensors
            assert!(active_sensors.contains(&"temp_01".to_string()));
            assert_eq!(readings_count, 0); // No readings yet
        } else {
            panic!("Expected status response");
        }

        // Test GetReading command
        let message = handler.create_command(Command::GetReading {
            sensor_id: "temp_01".to_string(),
        });
        let response = handler.process_command(message);

        if let MessagePayload::Response(Response::Reading { sensor_id, temperature, timestamp: _ }) = response.payload {
            assert_eq!(sensor_id, "temp_01");
            assert!((temperature - 23.5).abs() < 1.0); // Should be close to base temp (23.5) with some variation
        } else {
            panic!("Expected reading response");
        }

        // Test SetThreshold command
        let message = handler.create_command(Command::SetThreshold {
            sensor_id: "temp_01".to_string(),
            min_temp: 15.0,
            max_temp: 35.0,
        });
        let response = handler.process_command(message);

        if let MessagePayload::Response(Response::ThresholdSet { sensor_id, min_temp, max_temp }) = response.payload {
            assert_eq!(sensor_id, "temp_01");
            assert_eq!(min_temp, 15.0);
            assert_eq!(max_temp, 35.0);
        } else {
            panic!("Expected threshold set response");
        }
    }

    #[test]
    fn test_calibration() {
        let mut handler = TemperatureProtocolHandler::new();

        // Test calibration
        let message = handler.create_command(Command::Calibrate {
            sensor_id: "temp_01".to_string(),
            actual_temp: 25.0,
        });
        let response = handler.process_command(message);

        if let MessagePayload::Response(Response::CalibrationComplete { sensor_id, offset_adjustment }) = response.payload {
            assert_eq!(sensor_id, "temp_01");
            // The offset should be the difference between actual and measured temperature
            println!("Calibration offset: {}", offset_adjustment);
            assert!(offset_adjustment.abs() < 10.0); // Reasonable calibration offset
        } else {
            panic!("Expected calibration complete response");
        }
    }
}