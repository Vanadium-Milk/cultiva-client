#[macro_use] extern crate rust_i18n;
i18n!();

use std::error::Error;
use sudo::RunningAs;

mod setup;
mod db_client;
#[cfg(test)]
mod tests;
mod rest_client;

#[tokio::main]
async fn  main() -> Result<(), Box<dyn Error>> {
    if sudo::check() == RunningAs::User{
        panic!("{}", t!("no_root"));
    }
    setup::setup().await?;

    Ok( () )
}