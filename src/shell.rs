use crate::settings::Board;
use std::error::Error;
use std::io::ErrorKind::Unsupported;
use std::io::stderr;
use std::io::stdout;
use std::io::{Error as IoError, Write};
use std::process::{Command, Output, Stdio};

fn display_output(out: Output) -> Result<(), IoError> {
    stdout().write_all(&out.stdout)?;
    stderr().write_all(&out.stderr)?;
    Ok(())
}

pub fn install_arduino_cli() -> Result<(), IoError> {
    let script = Command::new("curl")
        .args([
            "-fsSL",
            "https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh",
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    let out = Command::new("sh")
        .env("BINDIR", "/usr/bin")
        .stdin(Stdio::from(script.stdout.unwrap()))
        .output()?;

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

pub fn get_board() -> Result<Board, Box<dyn Error>> {
    let boards = Command::new("arduino-cli")
        .args(["board", "list"])
        .output()?;

    let out = String::from_utf8(boards.stdout)?.trim().to_string();
    let lines: Vec<&str> = out.split("\n").collect();

    //Implement later handling for multiple arduino boards
    if lines.len() > 2 {
        return Err(Box::new(IoError::new(Unsupported, t!("board.multiple"))));
    }

    let attributes: Vec<&str> = lines[1].split(" ").collect();

    let curr_board = Board {
        name: attributes[9].to_string(),
        port: attributes[0].to_string(),
    };

    Ok(curr_board)
}
