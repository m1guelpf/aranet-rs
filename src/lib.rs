use btleplug::{
    api::{Central as _, Characteristic, Manager as _, Peripheral as _, ScanFilter},
    platform::{Adapter, Manager, Peripheral},
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::{io::Cursor, time::Duration};
use uuid::{uuid, Uuid};

const ADVERTISED_SERVICE: Uuid = uuid!("0000fce0-0000-1000-8000-00805f9b34fb");
const CURRENT_READINGS_CHARACTERISTIC: Uuid = uuid!("f0cd3001-95da-4f4b-9ac8-aa55d312af0c");

/// A connection to an Aranet4 device
pub struct Aranet4 {
    device: Peripheral,
    current_readings: Characteristic,
}

/// Errors that can occur when connecting to an Aranet4 device
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    /// Could not find a Bluetooth adapter
    #[error("Failed to find a Bluetooth adapter")]
    AdapterUnavaliable,

    /// Could not find an Aranet4 device before timeout
    #[error("Failed to find an Aranet4 device before timeout")]
    SearchTimeout,

    /// The specified characteristic was not found
    #[error("The characteristic {0} was not found")]
    CharacteristicNotFound(String),

    /// Other Bluetooth errors
    #[error(transparent)]
    BTLE(#[from] btleplug::Error),
}

/// Find an Aranet4 device and connect to it
pub async fn connect() -> Result<Aranet4, ConnectionError> {
    let manager = Manager::new().await.unwrap();

    let adapters = manager
        .adapters()
        .await
        .map_err(|_| ConnectionError::AdapterUnavaliable)?;

    let adapter = adapters
        .first()
        .ok_or(ConnectionError::AdapterUnavaliable)?;

    adapter
        .start_scan(ScanFilter {
            services: vec![ADVERTISED_SERVICE],
        })
        .await?;

    let device = tokio::select! {
        device = find_device(adapter) => device?,
        _ = tokio::time::sleep(Duration::from_secs(10)) => {
            return Err(ConnectionError::SearchTimeout)
        }
    };

    device.connect().await?;

    let chars = device.characteristics();
    let current_readings = chars
        .into_iter()
        .find(|c| c.uuid == CURRENT_READINGS_CHARACTERISTIC)
        .ok_or(ConnectionError::CharacteristicNotFound(
            CURRENT_READINGS_CHARACTERISTIC.to_string(),
        ))?;

    Ok(Aranet4 {
        device,
        current_readings,
    })
}

/// Data read from the Aranet4 device
#[derive(Debug)]
pub struct SensorData {
    // CO2 concentration in ppm
    pub co2: u16,
    // CO2 concentration status
    pub status: Semaphore,
    // Percentage of battery remaining
    pub battery: u8,
    // Percentage of relative humidity
    pub humidity: u8,
    // Atmospheric pressure in hPa
    pub pressure: u16,
    // Temperature in Celsius
    pub temperature: f32,
    // Measurement interval
    pub interval: Duration,
    // Time since last update
    pub since_last_update: Duration,
}

/// CO2 concentration status, as displayed by the device
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Semaphore {
    GREEN = 1,
    AMBER = 2,
    RED = 3,
}

impl From<u8> for Semaphore {
    fn from(value: u8) -> Self {
        match value {
            1 => Semaphore::GREEN,
            2 => Semaphore::AMBER,
            3 => Semaphore::RED,
            _ => panic!("invalid semaphore value"),
        }
    }
}

/// Errors that can occur when reading data from an Aranet4 device
#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    BTLE(#[from] btleplug::Error),
}

impl Aranet4 {
    /// Read data from the device
    pub async fn read_data(&self) -> Result<SensorData, DeviceError> {
        if !self.device.is_connected().await? {
            self.reconnect().await?;
        }

        let mut payload = Cursor::new(self.device.read(&self.current_readings).await?);

        let co2 = payload.read_u16::<LittleEndian>()?;
        let temperature = payload.read_u16::<LittleEndian>()? as f32 / 20.0;
        let pressure = payload.read_u16::<LittleEndian>()? / 10;
        let humidity = payload.read_u8()?;
        let battery = payload.read_u8()?;
        let status = payload.read_u8()?;
        let update_interval = payload.read_u16::<LittleEndian>()?;
        let since_last_update = payload.read_u16::<LittleEndian>()?;

        Ok(SensorData {
            co2,
            battery,
            humidity,
            pressure,
            temperature,
            status: Semaphore::from(status),
            interval: Duration::from_secs(update_interval as u64),
            since_last_update: Duration::from_secs(since_last_update as u64),
        })
    }

    /// Reconnect to the device
    pub async fn reconnect(&self) -> Result<(), DeviceError> {
        self.device.connect().await?;

        Ok(())
    }

    /// Disconnect from the device
    pub async fn disconnect(&self) -> Result<(), DeviceError> {
        self.device.disconnect().await?;

        Ok(())
    }
}

async fn find_device(adapter: &Adapter) -> Result<Peripheral, btleplug::Error> {
    loop {
        let peripherals = adapter.peripherals().await.unwrap();

        for peripheral in peripherals.into_iter() {
            let properties = peripheral.properties().await.unwrap().unwrap();
            let Some(name) = properties.local_name else {
                continue;
            };

            if name.starts_with("Aranet4") {
                return Ok(peripheral);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
