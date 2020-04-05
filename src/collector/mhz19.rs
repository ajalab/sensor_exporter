use std::error::Error;
use std::path::Path;
use std::time::Duration;

use prometheus::{IntCounter, IntGauge, Opts, Registry};

use crate::sensor::mhz19::MHZ19;

pub struct MHZ19Collector {
    sensor: MHZ19,
    interval: Duration,
    timeout: Duration,

    gauge_co2_ppm: IntGauge,
    counter_failure_total: IntCounter,
    counter_measure_total: IntCounter,
}

const GAUGE_CO2_PPM_NAME: &str = "mhz19_co2_ppm";
const GAUGE_CO2_PPM_HELP: &str = "co2 ppm";
const COUNTER_MEASURE_TOTAL_NAME: &str = "mhz19_measure_total";
const COUNTER_MEASURE_TOTAL_HELP: &str = "total count of measurement";
const COUNTER_FAILURE_TOTAL_NAME: &str = "mhz19_failure_total";
const COUNTER_FAILURE_TOTAL_HELP: &str = "total count of measurement failures";

impl MHZ19Collector {
    pub fn open<P>(
        path: P,
        registry: &Registry,
        interval: Duration,
        timeout: Duration,
    ) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let sensor = MHZ19::open(path)?;

        let gauge_co2_ppm = IntGauge::with_opts(Opts::new(GAUGE_CO2_PPM_NAME, GAUGE_CO2_PPM_HELP))?;
        let counter_measure_total = IntCounter::with_opts(Opts::new(
            COUNTER_MEASURE_TOTAL_NAME,
            COUNTER_MEASURE_TOTAL_HELP,
        ))?;
        let counter_failure_total = IntCounter::with_opts(Opts::new(
            COUNTER_FAILURE_TOTAL_NAME,
            COUNTER_FAILURE_TOTAL_HELP,
        ))?;

        registry.register(Box::new(gauge_co2_ppm.clone()))?;
        registry.register(Box::new(counter_measure_total.clone()))?;
        registry.register(Box::new(counter_failure_total.clone()))?;

        Ok(MHZ19Collector {
            sensor,
            interval,
            timeout,

            gauge_co2_ppm,
            counter_measure_total,
            counter_failure_total,
        })
    }

    pub async fn run(mut self) -> ! {
        loop {
            let result = tokio::time::timeout(self.timeout, self.sensor.measure()).await;
            if let Ok(Ok(v)) = result {
                self.gauge_co2_ppm.set(v as i64);
            } else {
                self.counter_failure_total.inc();
                let e: Box<dyn Error> = result.map_or_else(|e| e.into(), |r| r.unwrap_err().into());
                log::error!("fail to measure: {}", e);
            };
            self.counter_measure_total.inc();

            tokio::time::delay_for(self.interval).await;
        }
    }
}
