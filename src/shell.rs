use crate::settings::Board;
use std::error::Error;
use std::io::ErrorKind::{NotFound, Unsupported};
use std::io::stderr;
use std::io::stdout;
use std::io::{Error as IoError, Write};
use std::process::{Command, Output, Stdio};

fn display_output(out: Output) -> Result<(), IoError> {
    stdout().write_all(&out.stdout)?;
    stderr().write_all(&out.stderr)?;
    Ok(())
}

fn install_libraries() -> Result<(), IoError> {
    println!("Installing required sensor libraries...");

    //Insert the rest of the libraries in this function
    let out = Command::new("arduino-cli")
        .args(["lib", "install", "DHT sensor library"])
        .output()?;

    display_output(out)?;

    Ok(())
}

fn install_core(name: &str) -> Result<(), IoError> {
    println!("Installing required arduino cores...");

    let out = Command::new("arduino-cli")
        .args(["core", "install", name])
        .output()?;

    display_output(out)?;

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

    install_core("arduino:avr")?;
    install_libraries()?;

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

    if lines.len() <= 1 {
        return Err(Box::new(IoError::new(NotFound, t!("board.none"))));
    } else if lines.len() > 2 {
        //Implement later handling for multiple arduino boards
        return Err(Box::new(IoError::new(Unsupported, t!("board.multiple"))));
    }

    let attributes: Vec<&str> = lines[1].split(" ").collect();

    let curr_board = Board {
        name: attributes[9].to_string(),
        port: attributes[0].to_string(),
    };

    Ok(curr_board)
}

fn get_code_path(b_name: &str) -> String {
    let path = b_name.replace(":", "/");
    "/var/lib/cultiva/cultiva-microcontroller".to_owned() + "/" + &path
}

pub fn compile_arduino(
    board_name: &str,
    sensors_flag: u8,
    actuators_flag: u8,
) -> Result<(), IoError> {
    let path = get_code_path(board_name);

    let out = Command::new("arduino-cli")
        .args([
            "compile",
            "-b",
            board_name,
            "--build-property",
            &format!(
                "build.extra_flags=-DSENSORS={} -DACTUATORS={}",
                sensors_flag, actuators_flag
            ),
            &path,
        ])
        .output()?;

    dbg!(sensors_flag);
    dbg!(actuators_flag);

    display_output(out)?;
    Ok(())
}

pub fn upload_arduino(board_name: &str, port: &str) -> Result<(), IoError> {
    let path = get_code_path(board_name);

    let out = Command::new("arduino-cli")
        .args(["upload", &path, "-p", port, "-b", board_name])
        .output()?;

    display_output(out)?;
    Ok(())
}
