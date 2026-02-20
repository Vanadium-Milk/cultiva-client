mod serial;
mod socket_io;

use crate::db_client::{get_readings, insert_reading};
use crate::settings::load_conf;
use rust_socketio::{ClientBuilder, Payload, RawClient};
use serialport::SerialPort;
use std::error::Error;
use std::io::Error as IoError;
use std::io::ErrorKind::Deadlock;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

fn register_data(serial: Arc<Mutex<Box<dyn SerialPort>>>) -> IoError {
    //Polling loop with delay
    let mut cycle = Duration::from_secs(10);
    loop {
        //Adding sleep before the lock, so the mutex stays available
        sleep(cycle);
        cycle = Duration::from_secs(10);

        let locked = serial.lock();
        match locked {
            Ok(mut serial) => match serial::poll_sensors(&mut serial) {
                Ok(read) => match insert_reading(read) {
                    Ok(_) => {
                        println!("insert reading success");
                        cycle = Duration::from_mins(1);
                    }
                    Err(e) => {
                        eprintln!("{}: {} {}", t!("serial.insert_error"), e, t!("retry"));
                    }
                },
                Err(err) => {
                    eprintln!("{}: {} {}", t!("serial.input_error"), err, t!("retry"));
                }
            },
            Err(e) => {
                return IoError::new(Deadlock, format!("{} {}", t!("fatal"), e));
            }
        }
    }
}

pub(super) fn start_tasks() -> Result<(), Box<dyn Error>> {
    let config = load_conf()?;

    let port = Arc::new(Mutex::new(
        serialport::new(config.board.port, 9600)
            .timeout(Duration::from_secs(5))
            .open()?,
    ));

    //Sorry, IDK how to do this cleanly, here's two mutex copies lol
    let command_port = Arc::clone(&port);

    //Callback to pass the port value to the command handling function
    let activation_callback = move |payload: Payload, _| match payload {
        Payload::Text(text) => loop {
            let locked = command_port.lock();
            match locked {
                Ok(mut locked) => {
                    serial::send_command(&mut locked, text);
                    break;
                }
                Err(e) => {
                    eprintln!("Could not lock the serial port, retrying... {}", e);
                    sleep(Duration::from_secs(5));
                }
            }
        },
        _ => {
            eprintln!("Invalid payload received {:?}", payload);
        }
    };

    let query_callback = |payload: Payload, client: RawClient| match payload {
        Payload::Text(text) => loop {
            let data = get_readings(10);
            match data {
                Ok(readings) => {
                    match socket_io::send_readings(&client, text[0].as_str().unwrap(), readings) {
                        Ok(_) => {
                            println!("sent data xd");
                            break;
                        }
                        Err(e) => {
                            eprintln!("Error sending data: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Could not send readings, retrying... {}", e);
                }
            }
        },
        _ => {
            eprintln!("Invalid payload received {:?}", payload);
        }
    };

    //Initiate a socket.io connection
    let socket = ClientBuilder::new("http://localhost")
        .on("activate", activation_callback)
        .on("query", query_callback)
        .connect()?;

    //Authentication retry loop
    loop {
        sleep(Duration::from_secs(10));
        match socket_io::authenticate_connection(&socket) {
            Ok(_) => {
                println!("authentication success");
                break;
            }
            Err(e) => {
                eprint!("Could not authenticate connection: {}", e);
            }
        }
    }

    //Added delay to ensure serial connection is ready
    sleep(Duration::from_secs(5));
    register_data(port);

    Ok(())
}
