use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::{stderr, stdout, Error as ioError, ErrorKind, Write};
use std::process::Command;
use std::vec::Vec;
use std::process::Stdio;

#[derive(Deserialize, Serialize, Default)]
pub struct Settings {
    pub network: NetConf,
    pub physical_interface: IO,
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

//Convert indexes from menu selections to enums
impl TryFrom<&usize> for Sensors {
    type Error = ioError;
    fn try_from(value: &usize) -> Result<Self, ioError> {
        let res = match value {
            0 => Sensors::Thermometer,
            1 => Sensors::Hygrometer,
            2 => Sensors::SoilHygrometer,
            3 => Sensors::Luminometer,
            4 => Sensors::Co2,
            5 => Sensors::PH,
            _ => return Err(ioError::new(ErrorKind::InvalidInput, "Out of range value")),
        };
        Ok(res)
    }
}

impl TryFrom<&usize> for Actuators {
    type Error = ioError;
    fn try_from(value: &usize) -> Result<Self, ioError> {
        let res = match value {
            0 => Actuators::Irrigator,
            1 => Actuators::Heater,
            2 => Actuators::Lighting,
            3 => Actuators::UV,
            4 => Actuators::Shading,
            _ => return Err(ioError::new(ErrorKind::InvalidInput, "Out of range value")),
        };
        Ok(res)
    }
}

pub(super) fn save_conf(config: Settings) -> Result<(), Box<dyn Error>> {
    let content = toml::to_string(&config)?;
    fs::write("/etc/cultiva/cultiva.toml", content)?;

    Ok(())
}

pub(super) fn save_jwt(token: String) -> Result<(), ioError> {
    let token = Command::new("echo")
        .arg(token)
        .stdout(Stdio::piped())
        .spawn()?;

    let out = Command::new("systemd-creds")
        .args([
            "encrypt",
            "--name",
            "jwt",
            "/dev/stdin",
            "ciphertext.cred",
        ])
        .stdin(Stdio::from(token.stdout.unwrap()))
        .output()?;

    stdout().write_all(&out.stdout)?;
    stderr().write_all(&out.stderr)?;

    Ok(())
}

#[test]
fn test_save() -> Result<(), Box<dyn Error>> {
    let test_settings = Settings {
        network: NetConf { online: true },
        physical_interface: IO {
            sensors: vec![Sensors::Thermometer, Sensors::Hygrometer, Sensors::Co2],
            actuators: vec![Actuators::Irrigator, Actuators::Heater, Actuators::Lighting],
        },
    };
    save_conf(test_settings)?;

    Ok(())
}
