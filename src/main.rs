use std::error::Error;

mod db_client;
mod setup;
mod tests;

fn main() -> Result<(), Box<dyn Error>> {
    sudo::escalate_if_needed()?;
    setup::setup();
    Ok( () )
}