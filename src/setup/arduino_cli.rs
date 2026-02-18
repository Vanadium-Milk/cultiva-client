use crate::settings::Board;
use crate::shell::{display_output, execute_command};
use std::error::Error;
use std::io::Error as IoError;
use std::io::ErrorKind::{NotFound, Unsupported};
use std::process::{Command, Stdio};

fn install_libraries() -> Result<(), IoError> {
    println!("Installing required sensor libraries...");

    //Insert the rest of the libraries in this function
    execute_command("arduino-cli", &["lib", "install", "DHT sensor library"])?;
    Ok(())
}

fn install_core(name: &str) -> Result<(), IoError> {
    println!("Installing required arduino cores...");
    execute_command("arduino-cli", &["core", "install", name])?;
    Ok(())
}

fn get_code_path(b_name: &str) -> String {
    let path = b_name.replace(":", "/");
    "/var/lib/cultiva/cultiva-microcontroller".to_owned() + "/" + &path
}

pub(super) fn install_arduino_cli() -> Result<(), IoError> {
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

pub(super) fn get_board() -> Result<Board, Box<dyn Error>> {
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

pub(super) fn compile_sketch(
    board_name: &str,
    sensors_flag: u8,
    actuators_flag: u8,
) -> Result<(), IoError> {
    let path = get_code_path(board_name);

    execute_command(
        "arduino-cli",
        &[
            "compile",
            "-b",
            board_name,
            "--build-property",
            &format!(
                "build.extra_flags=-DSENSORS={} -DACTUATORS={}",
                sensors_flag, actuators_flag
            ),
            &path,
        ],
    )?;

    Ok(())
}

pub fn upload_sketch(board_name: &str, port: &str) -> Result<(), IoError> {
    let path = get_code_path(board_name);
    execute_command(
        "arduino-cli",
        &["upload", &path, "-p", port, "-b", board_name],
    )?;

    Ok(())
}

#[test]
fn test_get_boards() -> Result<(), Box<dyn Error>> {
    println!("Querying current board information...");
    dbg!(get_board()?);

    Ok(())
}
