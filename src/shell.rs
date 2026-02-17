use std::io::stderr;
use std::io::stdout;
use std::io::{Error as IoError, Write};
use std::process::{Command, Output, Stdio};

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

pub fn encrypt_key(key: &str, name: &str) -> Result<(), IoError> {
    let token = Command::new("echo")
        .arg(key)
        .stdout(Stdio::piped())
        .spawn()?;

    let out = Command::new("systemd-creds")
        .args(["encrypt", "--name", name, "/dev/stdin", "ciphertext.cred"])
        .stdin(Stdio::from(token.stdout.unwrap()))
        .output()?;

    display_output(out)?;

    Ok(())
}
