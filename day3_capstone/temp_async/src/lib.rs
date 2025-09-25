use std::time::Duration;
use tokio::time::{sleep, interval};
use tokio::sync::{mpsc, oneshot};
use temp_core::Temperature;
use temp_store::{TemperatureReading, TemperatureStore};

pub trait AsyncTemperatureSensor: Send {
    type Error: std::fmt::Debug + Send;

    async fn read_temperature(&mut self) -> Result<Temperature, Self::Error>;
    fn sensor_id(&self) -> &str;
}

pub struct AsyncMockSensor {
    id: String,
    temperature: f32,
    read_delay: Duration,
    fail_next: bool,
}

impl AsyncMockSensor {
    pub fn new(id: String, temperature: f32) -> Self {
        Self {
            id,
            temperature,
            read_delay: Duration::from_millis(100),
            fail_next: false,
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.read_delay = delay;
        self
    }

    pub fn set_temperature(&mut self, temp: f32) {
        self.temperature = temp;
    }

    pub fn fail_next_read(&mut self) {
        self.fail_next = true;
    }
}

#[derive(Debug)]
pub enum AsyncSensorError {
    ReadFailed,
    Timeout,
}

impl AsyncTemperatureSensor for AsyncMockSensor {
    type Error = AsyncSensorError;

    async fn read_temperature(&mut self) -> Result<Temperature, Self::Error> {
        sleep(self.read_delay).await;

        if self.fail_next {
            self.fail_next = false;
            return Err(AsyncSensorError::ReadFailed);
        }

        Ok(Temperature::new(self.temperature))
    }

    fn sensor_id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
pub enum MonitorCommand {
    SetInterval(Duration),
    GetStats(oneshot::Sender<Option<temp_store::TemperatureStats>>),
    GetLatest(oneshot::Sender<Option<TemperatureReading>>),
    Stop,
}

pub struct AsyncTemperatureMonitor {
    store: TemperatureStore,
    command_rx: mpsc::Receiver<MonitorCommand>,
    command_tx: mpsc::Sender<MonitorCommand>,
}

impl AsyncTemperatureMonitor {
    pub fn new(capacity: usize) -> Self {
        let (command_tx, command_rx) = mpsc::channel(32);
        Self {
            store: TemperatureStore::new(capacity),
            command_rx,
            command_tx,
        }
    }

    pub fn get_handle(&self) -> MonitorHandle {
        MonitorHandle {
            command_tx: self.command_tx.clone(),
        }
    }

    pub async fn run<S: AsyncTemperatureSensor>(&mut self, mut sensor: S, initial_interval: Duration) {
        let mut sample_interval = interval(initial_interval);

        loop {
            tokio::select! {
                _ = sample_interval.tick() => {
                    match sensor.read_temperature().await {
                        Ok(temp) => {
                            let reading = TemperatureReading::new(temp);
                            self.store.add_reading(reading);
                            println!("Temperature reading: {} from sensor {}", temp, sensor.sensor_id());
                        }
                        Err(e) => {
                            eprintln!("Failed to read temperature from {}: {:?}", sensor.sensor_id(), e);
                        }
                    }
                }

                command = self.command_rx.recv() => {
                    match command {
                        Some(MonitorCommand::SetInterval(new_interval)) => {
                            sample_interval = interval(new_interval);
                            println!("Changed sampling interval to {:?}", new_interval);
                        }
                        Some(MonitorCommand::GetStats(reply)) => {
                            let stats = self.store.calculate_stats();
                            let _ = reply.send(stats);
                        }
                        Some(MonitorCommand::GetLatest(reply)) => {
                            let latest = self.store.get_latest();
                            let _ = reply.send(latest);
                        }
                        Some(MonitorCommand::Stop) => {
                            println!("Stopping temperature monitor");
                            break;
                        }
                        None => {
                            println!("Command channel closed, stopping monitor");
                            break;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct MonitorHandle {
    command_tx: mpsc::Sender<MonitorCommand>,
}

impl MonitorHandle {
    pub async fn set_interval(&self, interval: Duration) -> Result<(), mpsc::error::SendError<MonitorCommand>> {
        self.command_tx.send(MonitorCommand::SetInterval(interval)).await
    }

    pub async fn get_stats(&self) -> Result<Option<temp_store::TemperatureStats>, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(MonitorCommand::GetStats(tx)).await?;
        Ok(rx.await?)
    }

    pub async fn get_latest(&self) -> Result<Option<TemperatureReading>, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(MonitorCommand::GetLatest(tx)).await?;
        Ok(rx.await?)
    }

    pub async fn stop(&self) -> Result<(), mpsc::error::SendError<MonitorCommand>> {
        self.command_tx.send(MonitorCommand::Stop).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn async_sensor_works() {
        let mut sensor = AsyncMockSensor::new("test".to_string(), 25.0);

        let reading = sensor.read_temperature().await.unwrap();
        assert_eq!(reading.celsius, 25.0);
        assert_eq!(sensor.sensor_id(), "test");
    }

    #[tokio::test]
    async fn async_sensor_respects_delay() {
        let mut sensor = AsyncMockSensor::new("test".to_string(), 25.0)
            .with_delay(Duration::from_millis(200));

        let start = std::time::Instant::now();
        let _reading = sensor.read_temperature().await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(190));
    }

    #[tokio::test]
    async fn async_sensor_can_fail() {
        let mut sensor = AsyncMockSensor::new("test".to_string(), 25.0);

        sensor.fail_next_read();
        let result = sensor.read_temperature().await;
        assert!(matches!(result, Err(AsyncSensorError::ReadFailed)));

        // Should work again
        let reading = sensor.read_temperature().await.unwrap();
        assert_eq!(reading.celsius, 25.0);
    }

    #[tokio::test]
    async fn monitor_handles_commands() {
        let mut monitor = AsyncTemperatureMonitor::new(10);
        let handle = monitor.get_handle();
        let sensor = AsyncMockSensor::new("test".to_string(), 20.0)
            .with_delay(Duration::from_millis(10));

        // Start monitor in background
        let monitor_task = tokio::spawn(async move {
            monitor.run(sensor, Duration::from_millis(100)).await;
        });

        // Wait a bit for some readings
        sleep(Duration::from_millis(250)).await;

        // Get stats
        let stats = handle.get_stats().await.unwrap();
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert!(stats.count >= 2);
        assert_eq!(stats.min.celsius, 20.0);

        // Get latest reading
        let latest = handle.get_latest().await.unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().temperature.celsius, 20.0);

        // Change interval
        handle.set_interval(Duration::from_millis(50)).await.unwrap();

        // Stop the monitor
        handle.stop().await.unwrap();

        // Wait for monitor to finish
        timeout(Duration::from_millis(500), monitor_task).await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn multiple_sensors_simulation() {
        // Simulate multiple sensors running concurrently
        let sensor1 = AsyncMockSensor::new("sensor1".to_string(), 20.0)
            .with_delay(Duration::from_millis(50));
        let sensor2 = AsyncMockSensor::new("sensor2".to_string(), 25.0)
            .with_delay(Duration::from_millis(75));

        let task1 = tokio::spawn(async move {
            let mut sensor = sensor1;
            for _ in 0..5 {
                let reading = sensor.read_temperature().await.unwrap();
                println!("Sensor 1: {}", reading);
                sleep(Duration::from_millis(100)).await;
            }
        });

        let task2 = tokio::spawn(async move {
            let mut sensor = sensor2;
            for _ in 0..5 {
                let reading = sensor.read_temperature().await.unwrap();
                println!("Sensor 2: {}", reading);
                sleep(Duration::from_millis(100)).await;
            }
        });

        let (r1, r2) = tokio::join!(task1, task2);
        r1.unwrap();
        r2.unwrap();
    }
}
