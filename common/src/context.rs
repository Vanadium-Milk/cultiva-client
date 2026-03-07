use config::{Config, ConfigError, File};
use std::collections::HashMap;
use std::error::Error;
use std::io::ErrorKind::UnexpectedEof;
use std::{fs, io};

pub fn set_context(context: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let content = toml::to_string(&context)?;
    fs::write("/etc/cultiva/context.toml", content)?;

    Ok(())
}

pub fn get_context() -> Result<HashMap<String, String>, ConfigError> {
    let path = "/etc/cultiva/context.toml";
    match fs::exists(path) {
        Ok(exists) => {
            if !exists {
                return Err(ConfigError::NotFound(t!("context.load_err").to_string()));
            }
        }
        Err(e) => return Err(ConfigError::Foreign(Box::from(e))),
    }

    let context = Config::builder()
        .add_source(File::with_name(path))
        .build()?
        .try_deserialize::<HashMap<String, String>>()?;

    if context.is_empty() {
        return Err(ConfigError::Foreign(Box::new(io::Error::new(
            UnexpectedEof,
            t!("context.no_data"),
        ))));
    }

    Ok(context)
}
