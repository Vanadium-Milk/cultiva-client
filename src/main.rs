#[macro_use]
extern crate rust_i18n;
i18n!();

use std::env::args;
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

    let args: Vec<String> = args().collect();
    if args.len() <= 1 {
        setup::load_conf().unwrap_or_else(|_| panic!("{}", t!("config.load_err")));
        todo!();
    } else if args[1] == "configure" {
        setup::setup().await?;
    } else {
        panic!("{}: {}", t!("arg_unknown"), args[1]);
    }

    Ok(())
}
