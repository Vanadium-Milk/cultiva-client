use std::env::var;
use std::error::Error;
use std::fs::read_to_string;
use std::io;
use std::io::{Write, stderr, stdout};
use std::process::{Command, Stdio};

pub fn get_jwt() -> Result<String, Box<dyn Error>> {
    let cred_dir = var("JWT")?;
    Ok(read_to_string(cred_dir)?.trim_end().to_owned())
}

pub fn save_jwt(token: String) -> Result<(), io::Error> {
    encrypt_key(&token, "CULTIVAJWT")
}

fn encrypt_key(key: &str, name: &str) -> Result<(), io::Error> {
    let token = Command::new("echo")
        .arg(key)
        .stdout(Stdio::piped())
        .spawn()?;

    let out = Command::new("systemd-creds")
        .args([
            "encrypt",
            "--name",
            name,
            "/dev/stdin",
            "/etc/cultiva/jwt.cred",
        ])
        .stdin(Stdio::from(token.stdout.unwrap()))
        .output()?;

    stdout().write_all(&out.stdout)?;
    stderr().write_all(&out.stderr)?;

    Ok(())
}
