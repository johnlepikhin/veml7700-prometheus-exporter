use anyhow::anyhow;
use linux_embedded_hal::I2cdev;
use veml6030::{SlaveAddr, Veml6030};

pub struct Get;

impl Get {
    pub fn get(config: crate::config::Config) -> anyhow::Result<()> {
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

        let white = sensor
            .read_white()
            .map_err(|_err| anyhow!("Cannot read white from sensor"))?;
        let lux = sensor
            .read_lux()
            .map_err(|_err| anyhow!("Cannot read LUX from sensor"))?;

        println!("white: {:2}, lux: {:2}", white, lux);

        Ok(())
    }
}
