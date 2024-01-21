use anyhow::anyhow;
use linux_embedded_hal::I2cdev;
use prometheus_exporter::prometheus::{
    core::{AtomicF64, GenericGauge},
    register_gauge,
};
use std::{cell::RefCell, sync::atomic::AtomicUsize};
use veml6030::{SlaveAddr, Veml6030};

pub struct Run {
    config: crate::config::Config,
    request_counter: AtomicUsize,
    sensor: RefCell<Veml6030<I2cdev>>,
    gauge_veml770_white: GenericGauge<AtomicF64>,
    gauge_veml770_lux: GenericGauge<AtomicF64>,
}

impl Run {
    pub fn new(config: crate::config::Config) -> anyhow::Result<Self> {
        let gauge_veml770_white = register_gauge!("veml7700_white", "VEML7700 abient light white")?;
        let gauge_veml770_lux = register_gauge!("veml7700_lux", "VEML7700 abient light LUX")?;

        let dev = I2cdev::new(&config.i2c_device).map_err(|err| {
            anyhow!(
                "Cannot initialize i2c device {:?}: {}",
                config.i2c_device,
                err
            )
        })?;
        let address = SlaveAddr::default();
        let mut sensor = Veml6030::new(dev, address);
        sensor
            .enable()
            .map_err(|_err| anyhow!("Cannot enable sensor"))?;

        Ok(Self {
            config,
            request_counter: AtomicUsize::new(0),
            sensor: RefCell::new(sensor),
            gauge_veml770_white,
            gauge_veml770_lux,
        })
    }

    fn process_request(&self) -> anyhow::Result<()> {
        let mut sensor = self.sensor.borrow_mut();
        self.gauge_veml770_white.set(
            sensor
                .read_white()
                .map_err(|_err| anyhow!("Cannot read white from sensor"))? as f64,
        );
        self.gauge_veml770_lux.set(
            sensor
                .read_lux()
                .map_err(|_err| anyhow!("Cannot read LUX from sensor"))? as f64,
        );

        Ok(())
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let exporter = prometheus_exporter::start(self.config.exporter_listen)?;

        loop {
            let _guard = exporter.wait_request();
            let request_id = self
                .request_counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            slog_scope::scope(
                &slog_scope::logger().new(slog::o!("request_id" => request_id)),
                || {
                    slog_scope::info!("Request started");
                    if let Err(err) = self.process_request() {
                        slog_scope::error!("Request failed: {err}");
                    }
                    slog_scope::info!("Request finished");
                },
            );
        }
    }
}
