#[macro_use] extern crate rust_i18n;
i18n!();

use std::error::Error;

mod setup;
mod db_client;
mod tests;

fn main() -> Result<(), Box<dyn Error>> {
    sudo::escalate_if_needed()?;
    setup::setup()?;

    Ok( () )
}