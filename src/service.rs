mod serial;
mod socket_io;

use crate::service::serial::BoardControl;
use crate::service::socket_io::{ResponseStatus, report_result};
use common::db_client::{get_readings, insert_reading};
use common::settings::load_conf;
use rust_socketio::{ClientBuilder, Payload, RawClient};
use std::error::Error;
use std::io::Error as IoError;
use std::io::ErrorKind::Deadlock;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

fn register_data(board: Arc<Mutex<BoardControl>>) -> IoError {
    //Polling loop with delay
    let mut cycle = Duration::from_secs(10);
    loop {
        //Adding sleep before the lock, so the mutex stays available
        sleep(cycle);
        cycle = Duration::from_secs(10);

        let locked = board.lock();
        match locked {
            Ok(mut locked_board) => match locked_board.poll_sensors() {
                Ok(read) => match insert_reading(read) {
                    Ok(_) => {
                        println!("{}", t!("serial.inserted"));
                        cycle = Duration::from_mins(1);
                    }
                    Err(e) => {
                        eprintln!("{}. {}", t!("serial.insert_error", error = e), t!("retry"));
                    }
                },
                Err(err) => {
                    eprintln!("{}, {}", t!("serial.input_error", error = err), t!("retry"));
                }
            },
            Err(e) => {
                return IoError::new(Deadlock, format!("{}", t!("error.fatal", error = e)));
            }
        }
    }
}

pub(super) fn start_tasks() -> Result<(), Box<dyn Error>> {
    let config = load_conf()?;

    let board = Arc::new(Mutex::new(BoardControl::new(
        serialport::new(config.board.port, 9600)
            .timeout(Duration::from_secs(5))
            .open()?,
    )));

    let arc_board = Arc::clone(&board);

    //Callback to pass the port value to the command handling function
    let activation_callback = move |payload: Payload, socket: RawClient| match payload {
        Payload::Text(text) => loop {
            let locked = arc_board.lock();
            match locked {
                Ok(mut locked) => {
                    let res_id = text[0].as_str().unwrap();
                    match locked.set_activation(&text[1]) {
                        Ok(_) => {
                            report_result(
                                socket,
                                res_id,
                                ResponseStatus::Success,
                                "Command performed successfully",
                            );
                        }
                        Err(e) => {
                            report_result(socket, res_id, ResponseStatus::Failed, &e.to_string())
                        }
                    }
                    break;
                }
                Err(e) => {
                    eprintln!("{}. {}", t!("serial.lock_error", error = e), t!("retry"));
                    sleep(Duration::from_secs(5));
                }
            }
        },
        _ => {
            eprintln!("{}: {:?}", t!("socket_io.payload_invalid"), payload);
        }
    };

    let query_callback = |payload: Payload, client: RawClient| match payload {
        Payload::Text(text) => loop {
            let data = get_readings(10);
            match data {
                Ok(readings) => {
                    match socket_io::send_readings(&client, text[0].as_str().unwrap(), readings) {
                        Ok(_) => {
                            println!("{}", t!("query.sent"));
                            break;
                        }
                        Err(e) => {
                            eprintln!("{}", t!("query.send_error", error = e));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}. {}", t!("query.retrieve_error", error = e), t!("retry"));
                }
            }
        },
        _ => {
            eprintln!("{}: {:?}", t!("socket_io.payload_invalid"), payload);
        }
    };

    //Initiate a socket.io connection
    let socket = ClientBuilder::new("http://localhost")
        .on("command", activation_callback)
        .on("query", query_callback)
        .connect()?;

    //Authentication retry loop
    loop {
        sleep(Duration::from_secs(10));
        match socket_io::authenticate_connection(&socket) {
            Ok(_) => {
                println!("{}", t!("socket_io.auth_success"));
                break;
            }
            Err(e) => {
                eprint!("{}. {}", t!("socket_io.auth_error", error = e), t!("retry"));
            }
        }
    }

    //Added delay to ensure serial connection is ready
    sleep(Duration::from_secs(5));
    register_data(board);

    Ok(())
}
