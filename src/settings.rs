use std::io::{ErrorKind, Error};
use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Board {
    pub name: String,
    pub port: String,
}

#[derive(Deserialize, Serialize)]
pub enum Sensors {
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

pub fn load_conf() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::with_name("/etc/cultiva/cultiva.toml"))
        .build()?
        .try_deserialize::<Settings>()?;

    Ok(settings)
}

//Convert indexes from menu selections to enums
impl TryFrom<&usize> for Sensors {
    type Error = Error;
    fn try_from(value: &usize) -> Result<Self, Error> {
        let res = match value {
            0 => Sensors::Thermometer,
            1 => Sensors::Hygrometer,
            2 => Sensors::SoilHygrometer,
            3 => Sensors::Luminometer,
            4 => Sensors::Co2,
            5 => Sensors::PH,
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