use crate::shell::encrypt_key;
use common::settings::Settings;
use std::error::Error;
use std::fs;
use std::io::Error as ioError;

pub(super) fn save_conf(config: Settings) -> Result<(), Box<dyn Error>> {
    let content = toml::to_string(&config)?;
    fs::write("/etc/cultiva/cultiva.toml", content)?;

    Ok(())
}

pub(super) fn save_jwt(token: String) -> Result<(), ioError> {
    encrypt_key(&token, "CULTIVAJWT")?;
    Ok(())
}

#[test]
fn test_save() -> Result<(), Box<dyn Error>> {
    use common::settings::{Actuators, Board, IO, NetConf, Sensors};

    let test_settings = Settings {
        network: NetConf { online: true },
        physical_interface: IO {
            sensors: vec![Sensors::Thermometer, Sensors::Hygrometer, Sensors::Co2],
            actuators: vec![Actuators::Irrigator, Actuators::Heater, Actuators::Lighting],
        },
        board: Board {
            name: "arduino:avr:uno".to_string(),
            port: "/dev/ttyACM0".to_string(),
        },
    };
    save_conf(test_settings)?;

    Ok(())
}
