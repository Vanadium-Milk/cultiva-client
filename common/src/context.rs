use config::{Config, File};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Error as IoError;
use std::io::ErrorKind::NotFound;

pub fn set_context(context: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let content = toml::to_string(&context)?;
    fs::write("/etc/cultiva/context.toml", content)?;

    Ok(())
}

pub fn get_context() -> Result<HashMap<String, String>, Box<dyn Error>> {
    if !fs::exists("/etc/cultiva/context.toml")? {
        return Err(Box::new(IoError::new(NotFound, t!("context.load_err"))));
    }
    let context = Config::builder()
        .add_source(File::with_name("/etc/cultiva/context.toml"))
        .build()?
        .try_deserialize::<HashMap<String, String>>()?;

    Ok(context)
}
