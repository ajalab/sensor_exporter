use std::time::Duration;

use sensor_exporter::sensor::mhz19::MHZ19;
use tokio::time::{delay_for, timeout};

const MHZ19_PATH: &str = "/dev/ttyS0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let mut mhz19 = MHZ19::open(MHZ19_PATH)
        .unwrap_or_else(|_| panic!(format!("failed to open {}", MHZ19_PATH)));

    loop {
        match timeout(Duration::from_millis(500), mhz19.measure()).await {
            Ok(Ok(v)) => println!("CO2: {} ppm", v),
            Ok(Err(e)) => log::error!("error: {}", e),
            Err(e) => log::error!("error: {}", e),
        };
        delay_for(Duration::from_millis(1000)).await;
    }
}
