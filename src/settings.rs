use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

#[derive(Deserialize, Serialize, Default)]
pub struct Settings {
    pub network: NetConf,
    pub physical_interface: IO,
    pub board: Board,
}
#[derive(Deserialize, Serialize, Default)]
pub struct NetConf {
    pub online: bool,
}

#[derive(Deserialize, Serialize, Default)]
pub struct IO {
    pub sensors: Vec<Sensors>,
    pub actuators: Vec<Actuators>,
}

pub struct IOFlags {
    pub sensors_flag: u8,
    pub actuators_flag: u8,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Board {
    pub name: String,
    pub port: String,
}

#[derive(Deserialize, Serialize)]
pub enum Sensors {
    DHT11,
    Thermometer,
    Hygrometer,
    SoilHygrometer,
    Luminometer,
    Co2,
    PH,
}

#[derive(Deserialize, Serialize)]
pub enum Actuators {
    Irrigator,
    Heater,
    Lighting,
    UV,
    Shading,
}

impl Settings {
    pub fn new() -> Self {
        Default::default()
    }
}

//Convert indexes from menu selections to enums
impl TryFrom<&usize> for Sensors {
    type Error = Error;
    fn try_from(value: &usize) -> Result<Self, Error> {
        let res = match value {
            0 => Sensors::DHT11,
            1 => Sensors::Thermometer,
            2 => Sensors::Hygrometer,
            3 => Sensors::SoilHygrometer,
            4 => Sensors::Luminometer,
            5 => Sensors::Co2,
            6 => Sensors::PH,
            _ => return Err(Error::new(ErrorKind::InvalidInput, "Out of range value")),
        };
        Ok(res)
    }
}

impl TryFrom<&usize> for Actuators {
    type Error = Error;
    fn try_from(value: &usize) -> Result<Self, Error> {
        let res = match value {
            0 => Actuators::Irrigator,
            1 => Actuators::Heater,
            2 => Actuators::Lighting,
            3 => Actuators::UV,
            4 => Actuators::Shading,
            _ => return Err(Error::new(ErrorKind::InvalidInput, "Out of range value")),
        };
        Ok(res)
    }
}

impl From<IO> for IOFlags {
    fn from(value: IO) -> Self {
        let mut ssum: u8 = 0;
        for s in value.sensors {
            match s {
                Sensors::DHT11 => ssum += 16,
                Sensors::SoilHygrometer => ssum += 8,
                Sensors::Luminometer => ssum += 4,
                Sensors::Co2 => ssum += 2,
                Sensors::PH => ssum += 1,
                _ => {}
            }
        }

        let mut asum: u8 = 0;
        for a in value.actuators {
            match a {
                Actuators::Irrigator => asum += 16,
                Actuators::Heater => asum += 8,
                Actuators::Lighting => asum += 4,
                Actuators::UV => asum += 2,
                Actuators::Shading => asum += 1,
            }
        }

        Self {
            sensors_flag: ssum,
            actuators_flag: asum,
        }
    }
}

pub fn load_conf() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::with_name("/etc/cultiva/cultiva.toml"))
        .build()?
        .try_deserialize::<Settings>()?;

    Ok(settings)
}
