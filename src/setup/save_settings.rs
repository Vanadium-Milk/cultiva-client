use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::Error as ioError;
use std::process::Command;

#[derive(Deserialize, Serialize, Default)]
pub struct Settings {
    pub network: NetConf,
}
#[derive(Deserialize, Serialize, Default)]
pub struct NetConf {
    pub online: bool,
}

impl Settings {
    pub fn new() -> Self {
        Default::default()
    }
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
