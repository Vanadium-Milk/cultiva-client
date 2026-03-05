use std::io::stderr;
use std::io::stdout;
use std::io::{Error as IoError, Write};
use std::process::{Command, Output};

pub fn display_output(out: Output) -> Result<(), IoError> {
    stdout().write_all(&out.stdout)?;
    stderr().write_all(&out.stderr)?;
    Ok(())
}

pub fn execute_command(command: &str, args: &[&str]) -> Result<(), IoError> {
    let out = Command::new(command).args(args).output()?;
    display_output(out)?;

    Ok(())
}
