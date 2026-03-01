use common::db_client::{Reading, insert_reading};
use common::settings::{Actuators, Sensors, load_conf};
use serde::{Deserialize, Serialize};
use serialport::SerialPort;
use std::collections::HashMap;
use std::error::Error;
use std::io::ErrorKind::{Deadlock, InvalidData, Unsupported};
use std::io::{Error as IoError, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    irrigator: Option<bool>,
    heater: Option<bool>,
    lighting: Option<bool>,
    uv: Option<bool>,
    shading: Option<bool>,
}

pub(super) struct BoardControl {
    port: Box<dyn SerialPort>,
    pub(super) state: ActivationState,
    pub(super) auto_modes: ActivationState,
}

#[derive(Default, Serialize)]
pub(super) struct ActivationState {
    irrigator: bool,
    heater: bool,
    lighting: bool,
    uv: bool,
    shading: bool,
}

pub(super) enum Modes {
    Active,
    Auto,
}

impl ActivationState {
    pub(super) fn new() -> Self {
        Default::default()
    }
}

impl BoardControl {
    pub(super) fn new(port: Box<dyn SerialPort>) -> Self {
        BoardControl {
            port,
            state: ActivationState::new(),
            //Default is everything on auto for an easier usage
            auto_modes: ActivationState {
                irrigator: true,
                heater: true,
                lighting: true,
                uv: true,
                shading: true,
            },
        }
    }

    pub(super) fn get_activation(&mut self, mode: Modes) -> HashMap<String, bool> {
        let mut data: HashMap<String, bool> = HashMap::new();

        match load_conf() {
            Ok(config) => {
                let board_data = match mode {
                    Modes::Active => &self.state,
                    Modes::Auto => &self.auto_modes,
                };

                for a in config.physical_interface.actuators {
                    match a {
                        Actuators::Irrigator => {
                            data.insert("irrigator".to_string(), board_data.irrigator);
                        }
                        Actuators::Heater => {
                            data.insert("heater".to_string(), board_data.heater);
                        }
                        Actuators::Lighting => {
                            data.insert("lighting".to_string(), board_data.lighting);
                        }
                        Actuators::UV => {
                            data.insert("uv".to_string(), board_data.uv);
                        }
                        Actuators::Shading => {
                            data.insert("shading".to_string(), board_data.shading);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }

        data
    }

    pub(super) fn set_auto_modes(
        &mut self,
        command: &serde_json::value::Value,
    ) -> Result<(), Box<dyn Error>> {
        let spec: Command = serde_json::from_value(command.clone())?;

        let auto = &mut self.auto_modes;
        //Change provided options, keep the ones unspecified as they are
        auto.irrigator = spec.irrigator.unwrap_or(auto.irrigator);
        auto.heater = spec.heater.unwrap_or(auto.heater);
        auto.lighting = spec.lighting.unwrap_or(auto.lighting);
        auto.uv = spec.uv.unwrap_or(auto.uv);
        auto.shading = spec.shading.unwrap_or(auto.shading);

        Ok(())
    }

    //Turn on or off the different actuators
    pub(super) fn set_activation(
        &mut self,
        command: &serde_json::value::Value,
    ) -> Result<(), Box<dyn Error>> {
        let spec: Command = serde_json::from_value(command.clone())?;

        let mut sum = 1;

        //Change provided options, keep the ones unspecified as they are
        self.state.irrigator = spec.irrigator.unwrap_or(self.state.irrigator);
        self.state.heater = spec.heater.unwrap_or(self.state.heater);
        self.state.lighting = spec.lighting.unwrap_or(self.state.lighting);
        self.state.uv = spec.uv.unwrap_or(self.state.uv);
        self.state.shading = spec.shading.unwrap_or(self.state.shading);

        if self.state.irrigator {
            sum += 16;
        }
        if self.state.heater {
            sum += 8;
        }
        if self.state.lighting {
            sum += 4;
        }
        if self.state.uv {
            sum += 2;
        }
        if self.state.shading {
            sum += 1;
        }

        //I will make a better encoder later, I'm just lazy rn
        let encoded = match char::from_digit(sum, 33) {
            Some(c) => c.to_string().to_uppercase(),
            None => {
                return Err(Box::new(std::io::Error::new(
                    Unsupported,
                    t!("error.over_limit", limit = 32, number = sum),
                )));
            }
        };

        let res = self.port.write_all(encoded.as_bytes());
        match res {
            Ok(_) => {
                println!("{}", t!("command.sent", command = encoded));
            }
            Err(e) => {
                println!("{}", t!("command.error", error = e));
                return Err(e.into());
            }
        }

        //Confirm command was received
        let mut buffer: Vec<u8> = vec![0; 1];
        match self.port.read_exact(buffer.as_mut_slice()) {
            Ok(_) => {
                let response = String::from_utf8(buffer);
                match response {
                    Err(e) => {
                        eprintln!("{}", t!("command.unchecked", error = e));
                    }
                    Ok(value) => {
                        if value != encoded {
                            eprintln!(
                                "{}",
                                t!("command.unmatched", sent = encoded, received = value)
                            );
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", t!("command.unchecked", error = e));
            }
        }
        self.port.flush()?;

        Ok(())
    }

    //Request sensor data and parse it as a reading
    pub(super) fn poll_sensors(&mut self) -> Result<Reading, Box<dyn Error>> {
        self.port.write_all("0".as_bytes())?;

        let mut serial_buf: Vec<u8> = vec![0; 64];
        //Arduino is quite slow, so it's best to give some margin for a response
        sleep(Duration::from_millis(100));
        self.port.read(serial_buf.as_mut_slice())?;
        self.port.flush()?;

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
                t!("serial.invalid_data"),
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
}

pub(super) fn register_data(board: Arc<Mutex<BoardControl>>) -> IoError {
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
                        cycle = Duration::from_mins(30);
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
