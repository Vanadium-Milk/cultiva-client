use crate::shell::{display_output, execute_command};
use common::settings::Board;
use std::error::Error;
use std::fs::{exists, read_to_string};
use std::io::Error as IoError;
use std::io::ErrorKind::NotFound;
use std::process::{Command, Stdio};

fn install_library(name: &str) -> Result<(), IoError> {
    println!("Installing required sensor libraries...");

    //Insert the rest of the libraries in this function
    execute_command("arduino-cli", &["lib", "install", name])?;
    Ok(())
}

pub(super) fn install_core(name: &str) -> Result<(), IoError> {
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

    Ok(())
}

pub(super) fn get_board() -> Result<Board, Box<dyn Error>> {
    let boards = Command::new("arduino-cli")
        .args(["board", "list"])
        .output()?;

    let out = String::from_utf8(boards.stdout)?.trim().to_string();
    let lines: Vec<&str> = out.split("\n").collect();

    //Helper function to locate arduino info
    let find_fqbn = |item: &&&str| item.contains(":");

    //Abbreviation for board.none error
    let no_board = Box::new(IoError::new(NotFound, t!("board.none")));

    //First line is the table headers
    if lines.len() <= 1 {
        return Err(no_board);
    }

    let info = if lines.len() == 2 {
        lines[1]
    } else {
        // I figured that the arduino cli often messes up and lists things that aren't arduino boards
        //Implement later handling for multiple arduino boards
        println!("{}", t!("board.multiple", device = lines[1]));
        lines.iter().find(find_fqbn).unwrap_or(&lines[1])
    };

    let attributes: Vec<&str> = info.split_whitespace().collect();

    //FQBN always have the : notation, otherwise it might be another type of device
    let full_board = attributes
        .iter()
        .find(|a| a.contains(":"))
        .ok_or(no_board)?;

    let curr_board = Board {
        name: full_board.to_string(),
        port: attributes[0].to_string(),
    };

    Ok(curr_board)
}

pub(super) fn compile_sketch(
    board_name: &str,
    sensors_flag: u8,
    actuators_flag: u8,
    invert_flag: u8,
) -> Result<(), IoError> {
    let path = get_code_path(board_name);

    if !exists(&path)? {
        return Err(IoError::new(
            NotFound,
            t!("board.unsupported", name = board_name),
        ));
    }

    //Install libraries defined in the arduino core
    let dep_file = read_to_string(path.clone() + "/dependencies.txt")?;
    let dependencies = dep_file.trim().split('\n').collect::<Vec<&str>>();

    for dep in dependencies {
        install_library(dep)?;
    }

    execute_command(
        "arduino-cli",
        &[
            "compile",
            "-b",
            board_name,
            "--build-property",
            &format!(
                "build.extra_flags=-DSENSORS={} -DACTUATORS={} -DINVERT={}",
                sensors_flag, actuators_flag, invert_flag
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

#[test]
fn test_compile_sketch() {
    compile_sketch("arduino:samd:mkrwifi1010", 0, 0, 0).unwrap();
}
