use crate::db_client::{Reading, insert_reading};
use crate::settings::{Sensors, load_conf};
use serialport::SerialPort;
use std::error::Error;
use std::io::ErrorKind::InvalidData;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

//Request sensor data and parse it as a reading
fn poll_sensors(connection: &mut Box<dyn SerialPort>) -> Result<Reading, Box<dyn Error>> {
    connection.write_all("0".as_bytes())?;

    let mut serial_buf: Vec<u8> = vec![0; 64];
    //Arduino is quite slow, so it's best to give some margin for a response
    sleep(Duration::from_millis(100));
    connection.read(serial_buf.as_mut_slice())?;

    let message = String::from_utf8(serial_buf)?;
    let mut data = message.split(",").collect::<Vec<&str>>();
    data.pop();

    //Check if input values correspond to the sensors specification
    let sensors = load_conf()?.physical_interface.sensors;
    let expect_len = sensors.len()
        + if sensors.contains(&Sensors::DHT11) {
            1
        } else {
            0
        };
    if expect_len != data.len() {
        return Err(Box::from(std::io::Error::new(
            InvalidData,
            t!("serial.invalid_input"),
        )));
    }

    //Consume all values in data while iterating, NOTE: This only works if the configuration sensors
    //appear in the same order as the output value
    let mut read = Reading::new();
    for s in sensors {
        match s {
            Sensors::DHT11 => {
                read.temperature = Some(data[0].parse::<f32>()?);
                read.air_humidity = Some(data[1].parse::<f32>()?);

                data.drain(0..2);
            }
            Sensors::SoilHygrometer => {
                read.soil_humidity = Some(data[0].parse::<f32>()?);
                data.remove(0);
            }
            Sensors::Luminometer => {
                read.luminosity = Some(data[0].parse::<f32>()?);
                data.remove(0);
            }
            Sensors::Co2 => {
                read.air_quality = Some(data[0].parse::<f32>()?);
                data.remove(0);
            }
            Sensors::PH => {}
            _ => {
                //Standalone Thermometer and hygrometer implementation pending
            }
        }
    }
    Ok(read)
}

fn register_data(serial: &mut Box<dyn SerialPort>) {
    loop {
        match poll_sensors(serial) {
            Ok(read) => match insert_reading(read) {
                Ok(_) => {
                    println!("insert reading success");
                    sleep(Duration::from_mins(1));
                }
                Err(e) => {
                    println!("{}: {} {}", t!("serial.insert_error"), e, t!("retry"));
                    sleep(Duration::from_secs(4));
                }
            },
            Err(err) => {
                println!("{}: {} {}", t!("serial.input_error"), err, t!("retry"));
                sleep(Duration::from_secs(4));
            }
        }
    }
}

pub(super) fn start_tasks() -> Result<(), Box<dyn Error>> {
    let config = load_conf()?;

    let mut port = serialport::new(config.board.port, 9600)
        .timeout(Duration::from_secs(5))
        .open()?;

    //Added delay to ensure connection is ready
    sleep(Duration::from_secs(5));

    tokio::spawn(async move { register_data(&mut port) });

    Ok(())
}
