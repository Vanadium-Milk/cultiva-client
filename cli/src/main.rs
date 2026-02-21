#[macro_use]
extern crate rust_i18n;
i18n!();

use std::env::args;
use std::error::Error;
use sudo::RunningAs;

mod setup;
mod shell;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if sudo::check() == RunningAs::User {
        panic!("{}", t!("no_root"));
    }

    let args: Vec<String> = args().collect();
    if args.len() <= 1 {
        println!("todo");
    } else if args[1] == "configure" {
        setup::setup().await?;
    } else if args[1] == "compile" {
        setup::compile_microcontroller()?;
    } else {
        panic!("{}: {}", t!("arg_unknown"), args[1]);
    }

    Ok(())
}
