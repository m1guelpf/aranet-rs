use std::fs;

#[tokio::main]
async fn main() {
    let device = aranet::connect().await.unwrap();

    let info = device.info().await.unwrap();
    let measurements = device.measurements().await.unwrap();

    fs::write(
        "aranet.txt",
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            info.manufacturer_name,
            info.model_number,
            info.serial_number,
            info.hardware_revision,
            info.firmware_revision,
            measurements.temperature,
            measurements.humidity,
            measurements.co2,
            measurements.pressure,
            measurements.battery
        ),
    )
    .expect("Unable to write file");
}
