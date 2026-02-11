#[macro_use]
extern crate rust_i18n;
i18n!();

use std::error::Error;
use sudo::RunningAs;

mod db_client;
mod rest_client;
mod setup;
#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if sudo::check() == RunningAs::User {
        panic!("{}", t!("no_root"));
    }
    setup::setup().await?;

    Ok(())
}
