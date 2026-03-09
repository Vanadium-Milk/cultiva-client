#[macro_use]
extern crate rust_i18n;
i18n!(fallback = "en");

use common::locales::match_locales;
use std::env::args;
use std::error::Error;
use std::io;
use std::io::ErrorKind::PermissionDenied;
use sudo::RunningAs;

mod setup;
mod shell;

fn sudo_or_error() -> Result<(), io::Error> {
    if sudo::check() == RunningAs::User {
        Err(io::Error::new(PermissionDenied, t!("no_root")))?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let locale = match_locales().unwrap_or("en".to_string());
    rust_i18n::set_locale(&locale);

    let args: Vec<String> = args().collect();
    if args.len() <= 1 || args[1] == "--help" || args[1] == "-h"{
        println!("{}", t!("usage"));
    } else if args[1] == "configure" {
        sudo_or_error()?;
        setup::setup().await?;
    } else if args[1] == "compile" {
        sudo_or_error()?;
        setup::compile_microcontroller()?;
    }
    else {
        println!("{}", t!("arg_unknown", arg = args[1]));
        println!("{}", t!("usage"));
    }

    Ok(())
}
