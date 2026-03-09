#[macro_use]
extern crate rust_i18n;
i18n!(fallback = "en");

use common::locales::match_locales;
use std::error::Error;
use std::io;
use std::io::ErrorKind::PermissionDenied;
use sudo::RunningAs;

mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    rust_i18n::set_locale(&match_locales().unwrap_or("en".to_string()));

    if sudo::check() == RunningAs::User {
        Err(io::Error::new(PermissionDenied, t!("no_root")))?;
    }

    service::start_tasks().await?;

    Ok(())
}
