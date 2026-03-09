mod arduino_cli;
mod save_settings;

use crate::setup::arduino_cli::install_core;
use crate::setup::save_settings::save_conf;
use common::credentials::save_jwt;
use common::db_client::create_tables;
use common::rest_client::{Auth, Output, login_account, register_account};
use common::settings::{Actuators, Board, IOFlags, Sensors, Settings, load_conf};
use dialoguer::{Confirm, Input, MultiSelect, Password, Select};
use git2::Repository;
use std::error::Error;
use std::io;
use std::io::ErrorKind::Interrupted;

async fn register_loop() -> Result<(), Box<dyn Error>> {
    println!("{}", t!("login.disclaimer"));

    //Set email loop
    loop {
        let email: String = Input::new().with_prompt(t!("login.user")).interact_text()?;
        //Password re-attempt loop
        loop {
            let password: String = Password::new().with_prompt(t!("login.pass")).interact()?;
            if password
                == Password::new()
                    .with_prompt(t!("login.pass_confirm"))
                    .interact()?
            {
                let username: String =
                    Input::new().with_prompt(t!("login.name")).interact_text()?;

                //User registration
                let res = register_account(&email, &password, &username).await;
                match res {
                    Ok(r) => {
                        if r.status().is_success() {
                            //User register successful
                            println!("{}", t!("login.acc_created"));
                            return Ok(());
                        } else {
                            //Register failed either bad email or account already registered
                            println!(
                                "{}: {}",
                                t!("login.register_fail"),
                                r.json::<Output>().await?.message
                            );
                            break;
                        }
                    }
                    Err(e) => {
                        println!("{}. {}", e, t!("login.no_connection"));
                        break;
                    }
                }
            } else {
                println!("{}", t!("login.pass_match_fail"));
            }
        }
    }
}

async fn login_loop() -> Result<(), Box<dyn Error>> {
    loop {
        let email: String = Input::new().with_prompt(t!("login.user")).interact_text()?;

        let password: String = Password::new().with_prompt(t!("login.pass")).interact()?;

        let res = login_account(&email, &password).await;
        match res {
            Ok(r) => {
                if r.status().is_success() {
                    //User login successful
                    println!("{}", t!("login.success"));
                    save_jwt(r.json::<Auth>().await?.token)?;
                    return Ok(());
                } else {
                    //Incorrect credentials
                    println!(
                        "{}: {}",
                        t!("login.log_failed"),
                        r.json::<Output>().await?.message
                    );
                }
            }
            Err(e) => {
                println!("{}. {}", e, t!("login.no_connection"));
            }
        }
    }
}

pub(super) fn compile_microcontroller() -> Result<(), Box<dyn Error>> {
    let config = load_conf()?;

    //Clone microcontroller source code repo
    println!("{}", t!("board.source_code"));
    let url = "https://github.com/Vanadium-Milk/cultiva-microcontroller";
    let path = "/var/lib/cultiva/cultiva-microcontroller/";

    //Ideally this would be a fetch-pull operation, but this ensures no modifications remain
    if std::fs::exists(path)? {
        std::fs::remove_dir_all(path)?;
    }
    Repository::clone(url, "/var/lib/cultiva/cultiva-microcontroller")?;

    println!("{}", t!("board.compile", core = config.board.name));
    let flags: IOFlags = config.physical_interface.into();

    arduino_cli::compile_sketch(
        &config.board.name,
        flags.sensors_flag,
        flags.actuators_flag,
        flags.inverted_flag,
    )?;
    arduino_cli::upload_sketch(&config.board.name, &config.board.port)?;

    Ok(())
}

pub(super) async fn setup() -> Result<(), Box<dyn Error>> {
    println!("{}", t!("setup_ini"));

    if load_conf().is_ok() && !Confirm::new().with_prompt(t!("config.found")).interact()? {
        //User canceled setup, early exit
        return Ok(());
    }

    let mut configuration = Settings::new();

    //Confirm selection loop
    loop {
        let online = Select::new()
            .with_prompt(t!("online.prompt"))
            .items(vec![t!("online.official"), t!("online.unofficial")])
            .interact()?;

        configuration.network.online = online == 0;

        if configuration.network.online {
            let register = Select::new()
                .with_prompt(t!("login.registration"))
                .items(vec![t!("login.no_acc"), t!("login.acc")])
                .interact()?;

            if register == 0 {
                register_loop().await?;
                break;
            } else {
                login_loop().await?;
                break;
            }
        } else if Confirm::new()
            .with_prompt(t!("online.confirm_prompt"))
            .interact()?
        {
            break;
        }
    }

    let sensors = MultiSelect::new()
        .with_prompt(t!("sensors.set_sensors"))
        .items(vec![
            t!("sensors.dht11"),
            t!("sensors.therm"),
            t!("sensors.hygro"),
            t!("sensors.soil_hygro"),
            t!("sensors.lumin"),
            t!("sensors.co2"),
            t!("sensors.ph"),
        ])
        .interact()?;
    configuration.physical_interface.sensors = sensors
        .iter()
        .map(|val| val.try_into())
        .collect::<Result<Vec<Sensors>, io::Error>>()?;

    let act_items = MultiSelect::new().items(vec![
        t!("actuators.water"),
        t!("actuators.heat"),
        t!("actuators.light"),
        t!("actuators.uv"),
        t!("actuators.shade"),
    ]);

    let actuators = act_items
        .clone()
        .with_prompt(t!("actuators.set_act"))
        .interact()?;
    configuration.physical_interface.actuators = actuators
        .iter()
        .map(|val| val.try_into())
        .collect::<Result<Vec<Actuators>, io::Error>>()?;

    //Invert output of certain actuators with this setting
    let invert = act_items
        .with_prompt(t!("actuators.set_invert"))
        .interact()?;
    configuration.physical_interface.inverted = invert
        .iter()
        .map(|val| val.try_into())
        .collect::<Result<Vec<Actuators>, io::Error>>()?;

    if !Confirm::new()
        .with_prompt(t!("board.download"))
        .interact()?
    {
        return Err(Box::new(io::Error::new(Interrupted, t!("board.decline"))));
    }
    arduino_cli::install_arduino_cli()?;

    //Enter manual mode if the arduino board wasn't detected
    configuration.board =
        arduino_cli::get_board().or_else(|e| -> Result<Board, Box<dyn Error>> {
            eprintln!("{}", e);
            println!("{}", t!("board.manual_set.prompt"));
            loop {
                let port: String = Input::new()
                    .with_prompt(t!("board.manual_set.port"))
                    .interact_text()?;

                if !port.starts_with("/dev/") {
                    eprintln!("{}", t!("board.manual_set.invalid_input"));
                    continue;
                }

                let name: String = Input::new()
                    .with_prompt(t!("board.manual_set.board"))
                    .interact()?;

                let Some(core) = name.rsplit_once(":") else {
                    eprintln!("{}", t!("board.manual_set.invalid_input"));
                    continue;
                };
                install_core(core.0)?;

                return Ok(Board { port, name });
            }
        })?;

    println!("{}", t!("config.saving"));
    save_conf(configuration)?;

    println!("{}", t!("db_setup"));
    create_tables()?;

    if Confirm::new()
        .with_prompt(t!("board.compile_prompt"))
        .interact()?
    {
        compile_microcontroller()?;
    }

    println!("{}", t!("setup_complete"));

    Ok(())
}
