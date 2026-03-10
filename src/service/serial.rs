use common::db_client::Reading;
use common::settings::{Actuators, Sensors, load_conf};
use common::state_handling::ActivationState;
use serialport::SerialPort;
use std::collections::HashMap;
use std::error::Error;
use std::io::ErrorKind::{InvalidData, Unsupported};
use std::io::{Read, Write};
use std::thread::sleep;
use std::time::Duration;

pub(super) struct BoardControl {
    port: Box<dyn SerialPort>,
    pub(super) state: ActivationState,
    pub(super) auto_modes: ActivationState,
}

pub(super) enum Modes {
    Active,
    Auto,
}

impl BoardControl {
    pub(super) fn new(port: Box<dyn SerialPort>) -> Self {
        //Set only supported actuators, otherwise None
        let mut auto = ActivationState::new();
        let mut active = ActivationState::new();
        if let Ok(config) = load_conf() {
            for a in config.physical_interface.actuators {
                match a {
                    Actuators::Irrigator => {
                        auto.irrigator = Some(true);
                        active.irrigator = Some(false);
                    }
                    Actuators::Heater => {
                        auto.heater = Some(true);
                        active.heater = Some(false);
                    }
                    Actuators::Lighting => {
                        auto.lighting = Some(true);
                        active.lighting = Some(false);
                    }
                    Actuators::UV => {
                        auto.uv = Some(true);
                        active.uv = Some(false);
                    }
                    Actuators::Shading => {
                        auto.shading = Some(true);
                        active.shading = Some(false);
                    }
                }
            }
        }

        BoardControl {
            port,
            state: active,
            auto_modes: auto,
        }
    }

    pub(super) fn get_activation(&self, mode: Modes) -> HashMap<String, bool> {
        match mode {
            Modes::Active => self.state.into(),
            Modes::Auto => self.auto_modes.into(),
        }
    }

    //Changes the state only for spec values that contain Some()
    fn mutate_to_spec(state: &mut ActivationState, spec: ActivationState) {
        //I'm so sorry for this abomination, I wanted to do it the cool way, but the project is due
        //for 3 days
        state.irrigator = state.irrigator.and(spec.irrigator.or(state.irrigator));
        state.heater = state.heater.and(spec.heater.or(state.heater));
        state.lighting = state.lighting.and(spec.lighting.or(state.lighting));
        state.uv = state.uv.and(spec.uv.or(state.uv));
        state.shading = state.shading.and(spec.shading.or(state.shading));
    }

    pub(super) fn set_auto_modes(
        &mut self,
        command: ActivationState,
    ) -> Result<(), Box<dyn Error>> {
        Self::mutate_to_spec(&mut self.auto_modes, command);

        Ok(())
    }

    //Turn on or off the different actuators
    pub(super) fn set_activation(
        &mut self,
        command: ActivationState,
    ) -> Result<(), Box<dyn Error>> {
        let mut sum = 1;

        Self::mutate_to_spec(&mut self.state, command);

        if self.state.irrigator.is_some_and(|x| x) {
            sum += 16;
        }
        if self.state.heater.is_some_and(|x| x) {
            sum += 8;
        }
        if self.state.lighting.is_some_and(|x| x) {
            sum += 4;
        }
        if self.state.uv.is_some_and(|x| x) {
            sum += 2;
        }
        if self.state.shading.is_some_and(|x| x) {
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
                println!("{}", t!("serial.command.sent", command = encoded));
            }
            Err(e) => {
                println!("{}", t!("serial.command.error", error = e));
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
                        eprintln!("{}", t!("serial.command.unchecked", error = e));
                    }
                    Ok(value) => {
                        if value != encoded {
                            eprintln!(
                                "{}",
                                t!("serial.command.unmatched", sent = encoded, received = value)
                            );
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", t!("serial.command.unchecked", error = e));
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
