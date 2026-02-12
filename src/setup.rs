mod save_settings;

use crate::db_client::create_tables;
use crate::rest_client::{Auth, Output, login_account, register_account};
use crate::setup::save_settings::{Settings, save_conf, save_jwt};
use config::{Config, ConfigError, File};
use dialoguer::{Confirm, Input, Password, Select};
use std::error::Error;

i18n!();

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

pub(super) fn load_conf() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::with_name("/etc/cultiva/cultiva.toml"))
        .build()?
        .try_deserialize::<Settings>()?;

    Ok(settings)
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

    println!("{}", t!("config.saving"));
    save_conf(configuration)?;

    println!("{}", t!("db_setup"));

    let created = create_tables();
    if let Err(e) = created {
        panic!("{}", t!("db_panic", error = e));
    }

    Ok(())
}
