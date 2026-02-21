use common::db_client::Reading;
use common::settings::{Sensors, load_conf};
use serde_json::Value;
use serialport::SerialPort;
use std::error::Error;
use std::io::ErrorKind::InvalidData;
use std::io::{Read, Write};
use std::thread::sleep;
use std::time::Duration;

//Request sensor data and parse it as a reading
pub(super) fn poll_sensors(
    connection: &mut Box<dyn SerialPort>,
) -> Result<Reading, Box<dyn Error>> {
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

pub(super) fn send_command(connection: &mut Box<dyn SerialPort>, command: Vec<Value>) {
    let string = command[0].to_string();
    let res = connection.write_all(string.as_bytes());
    match res {
        Ok(_) => {
            println!("Sent command: {}", string);
        }
        Err(e) => {
            println!("Error sending command: {}", e);
        }
    }
}
