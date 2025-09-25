use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use temp_core::Temperature;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TemperatureReading {
    pub temperature: Temperature,
    pub timestamp: u64,
}

impl TemperatureReading {
    pub fn new(temperature: Temperature) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self { temperature, timestamp }
    }

    pub fn with_timestamp(temperature: Temperature, timestamp: u64) -> Self {
        Self { temperature, timestamp }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemperatureStats {
    pub min: Temperature,
    pub max: Temperature,
    pub average: Temperature,
    pub count: usize,
}

pub struct TemperatureStore {
    readings: Arc<Mutex<Vec<TemperatureReading>>>,
    capacity: usize,
}

impl TemperatureStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            readings: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            capacity,
        }
    }

    pub fn add_reading(&self, reading: TemperatureReading) {
        let mut readings = self.readings.lock().unwrap();

        if readings.len() >= self.capacity {
            readings.remove(0);
        }

        readings.push(reading);
    }

    pub fn get_latest(&self) -> Option<TemperatureReading> {
        let readings = self.readings.lock().unwrap();
        readings.last().copied()
    }

    pub fn get_all(&self) -> Vec<TemperatureReading> {
        let readings = self.readings.lock().unwrap();
        readings.clone()
    }

    pub fn calculate_stats(&self) -> Option<TemperatureStats> {
        let readings = self.readings.lock().unwrap();

        if readings.is_empty() {
            return None;
        }

        let mut min_temp = readings[0].temperature.celsius;
        let mut max_temp = readings[0].temperature.celsius;
        let mut sum = 0.0;

        for reading in readings.iter() {
            let temp = reading.temperature.celsius;
            if temp < min_temp {
                min_temp = temp;
            }
            if temp > max_temp {
                max_temp = temp;
            }
            sum += temp;
        }

        let average = sum / readings.len() as f32;

        Some(TemperatureStats {
            min: Temperature::new(min_temp),
            max: Temperature::new(max_temp),
            average: Temperature::new(average),
            count: readings.len(),
        })
    }

    pub fn get_stats(&self) -> TemperatureStats {
        self.calculate_stats().unwrap_or(TemperatureStats {
            min: Temperature::new(0.0),
            max: Temperature::new(0.0),
            average: Temperature::new(0.0),
            count: 0,
        })
    }

    pub fn reading_count(&self) -> usize {
        self.len()
    }

    pub fn get_recent_readings(&self, count: usize) -> Vec<TemperatureReading> {
        let readings = self.readings.lock().unwrap();
        let start_index = if readings.len() > count {
            readings.len() - count
        } else {
            0
        };
        readings[start_index..].to_vec()
    }

    pub fn clear(&self) {
        let mut readings = self.readings.lock().unwrap();
        readings.clear();
    }

    pub fn len(&self) -> usize {
        let readings = self.readings.lock().unwrap();
        readings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clone_handle(&self) -> Self {
        Self {
            readings: Arc::clone(&self.readings),
            capacity: self.capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn store_basic_operations() {
        let store = TemperatureStore::new(5);

        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert!(store.get_latest().is_none());
        assert!(store.calculate_stats().is_none());

        let reading = TemperatureReading::new(Temperature::new(20.0));
        store.add_reading(reading);

        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());

        let latest = store.get_latest().unwrap();
        assert_eq!(latest.temperature.celsius, 20.0);
    }

    #[test]
    fn store_circular_buffer() {
        let store = TemperatureStore::new(3);

        // Add more readings than capacity
        for i in 0..5 {
            let reading = TemperatureReading::new(Temperature::new(i as f32 * 10.0));
            store.add_reading(reading);
        }

        assert_eq!(store.len(), 3);

        let readings = store.get_all();
        assert_eq!(readings.len(), 3);

        // Should contain temperatures 20.0, 30.0, 40.0 (the last 3)
        assert_eq!(readings[0].temperature.celsius, 20.0);
        assert_eq!(readings[1].temperature.celsius, 30.0);
        assert_eq!(readings[2].temperature.celsius, 40.0);
    }

    #[test]
    fn store_statistics() {
        let store = TemperatureStore::new(10);

        let temps = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        for temp in temps {
            let reading = TemperatureReading::new(Temperature::new(temp));
            store.add_reading(reading);
        }

        let stats = store.calculate_stats().unwrap();
        assert_eq!(stats.min.celsius, 10.0);
        assert_eq!(stats.max.celsius, 50.0);
        assert_eq!(stats.average.celsius, 30.0);
        assert_eq!(stats.count, 5);
    }

    #[test]
    fn store_thread_safety() {
        let store = TemperatureStore::new(100);
        let store1 = store.clone_handle();
        let store2 = store.clone_handle();

        let handle1 = thread::spawn(move || {
            for i in 0..50 {
                let reading = TemperatureReading::new(Temperature::new(i as f32));
                store1.add_reading(reading);
            }
        });

        let handle2 = thread::spawn(move || {
            for i in 50..100 {
                let reading = TemperatureReading::new(Temperature::new(i as f32));
                store2.add_reading(reading);
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        assert_eq!(store.len(), 100);
        let stats = store.calculate_stats().unwrap();
        assert_eq!(stats.count, 100);
        assert_eq!(stats.min.celsius, 0.0);
        assert_eq!(stats.max.celsius, 99.0);
    }

    #[test]
    fn temperature_reading_creation() {
        let temp = Temperature::new(25.0);
        let reading = TemperatureReading::new(temp);

        assert_eq!(reading.temperature.celsius, 25.0);
        assert!(reading.timestamp > 0);

        let custom_reading = TemperatureReading::with_timestamp(temp, 1234567890);
        assert_eq!(custom_reading.timestamp, 1234567890);
    }
}
