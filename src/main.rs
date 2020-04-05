use std::error::Error;
use std::time::Duration;

use prometheus::{Encoder, Registry, TextEncoder};
use warp::{
    http::{Response, StatusCode},
    Filter,
};

use sensor_exporter::collector::MHZ19Collector;

const MHZ19_PATH: &str = "/dev/ttyS0";
const MEAURE_INTERVAL_MILLIS: u64 = 1000;
const MEAURE_TIMEOUT_MILLIS: u64 = 100;
const PORT: u16 = 9750;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let registry = Registry::new();
    let mhz19_collector = MHZ19Collector::open(
        MHZ19_PATH,
        &registry,
        Duration::from_millis(MEAURE_INTERVAL_MILLIS),
        Duration::from_millis(MEAURE_TIMEOUT_MILLIS),
    )
    .expect("failed to open");

    let routes = warp::path("metrics").map(move || {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_family = registry.gather();

        encoder
            .encode(&metric_family, &mut buffer)
            .map_err(|e| e.into())
            .and_then(|_| String::from_utf8(buffer).map_err(|e| e.into()))
            .map_or_else(
                |e: Box<dyn Error>| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(e.to_string())
                },
                |s| Response::builder().status(StatusCode::OK).body(s),
            )
    });

    tokio::spawn(mhz19_collector.run());
    warp::serve(routes).run(([127, 0, 0, 1], PORT)).await;

    Ok(())
}
