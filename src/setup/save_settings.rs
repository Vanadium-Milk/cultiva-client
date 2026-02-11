use config::File;
use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::Error as ioError;
use std::process::Command;

#[derive(Deserialize, Serialize, Default)]
pub(super) struct Settings {
    pub(super) network: NetConf,
}
#[derive(Deserialize, Serialize, Default)]
pub(super) struct NetConf {
    pub(super) online: bool,
}

impl NetConf {
    fn new() -> Self {
        Default::default()
    }
}

impl Settings {
    fn new() -> Self {
        Default::default()
    }
}

pub(super) fn load_conf() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::with_name("/etc/cultiva/cultiva.toml"))
        .build()?
        .try_deserialize::<Settings>()?;

    Ok(settings)
}

pub(super) fn save_conf(config: Settings) -> Result<(), Box<dyn Error>> {
    let content = toml::to_string(&config)?;
    fs::write("/etc/cultiva/cultiva.toml", content)?;

    Ok(())
}

pub(super) fn save_jwt(token: String) -> Result<(), ioError> {
    Command::new("echo")
        .arg(token)
        .args([
            "|",
            "systemd-creds",
            "encrypt",
            "--name",
            "jwt",
            "/dev/stdin",
            "ciphertext.cred",
        ])
        .output()?;

    Ok(())
}

#[test]
fn test_save() -> Result<(), Box<dyn Error>> {
    let test_settings = Settings {
        network: NetConf { online: true },
    };
    save_conf(test_settings)?;

    Ok(())
}
