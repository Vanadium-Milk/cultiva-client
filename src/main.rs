#[macro_use] extern crate rust_i18n;
i18n!();

use std::error::Error;
use std::env::var;

mod setup;
mod db_client;
#[cfg(test)]
mod tests;
mod rest_client;

#[tokio::main]
async fn  main() -> Result<(), Box<dyn Error>> {
    sudo::escalate_if_needed()?;
    setup::setup().await?;

    Ok( () )
}