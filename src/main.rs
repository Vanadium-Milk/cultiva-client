#[macro_use]
extern crate rust_i18n;
i18n!();

use std::error::Error;
use sudo::RunningAs;

mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if sudo::check() == RunningAs::User {
        panic!("{}", t!("no_root"));
    }

    service::start_tasks().await?;

    Ok(())
}
