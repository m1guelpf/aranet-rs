# aranet-rs

> Read current measurements from an Aranet4 device in Rust.

[![crates.io](https://img.shields.io/crates/v/aranet.svg)](https://crates.io/crates/aranet)
[![download count badge](https://img.shields.io/crates/d/aranet.svg)](https://crates.io/crates/aranet)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/aranet)

## Usage

```rust
let device = aranet::connect().await?;

let measurements = device.measurements().await?;

dbg!(measurements);
// [src/main.rs:6] measurements = SensorData {
//   со2: 962,
//   battery: 76,
//   humidity: 49,
//   status: GREEN,
//   pressure: 1017,
//   interval: 300s,
//   temperature: 25.75,
//   since_last_update: 127s,
// }
```

Refer to the [documentation on docs.rs](https://docs.rs/aranet) for detailed usage instructions.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
