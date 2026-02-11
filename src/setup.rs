use crate::db_client;
use crate::rest_client;
use dialoguer;
use std::error::Error;

i18n!();

async fn register_loop() -> Result<(), Box<dyn Error>> {
    println!("{}", t!("login.disclaimer"));

    //Set email loop
    loop {
        let email: String = dialoguer::Input::new()
            .with_prompt(t!("login.user"))
            .interact_text()?;
        //Password re-attempt loop
        loop {
            let password: String = dialoguer::Password::new()
                .with_prompt(t!("login.pass"))
                .interact()?;
            if password
                == dialoguer::Password::new()
                    .with_prompt(t!("login.pass_confirm"))
                    .interact()?
            {
                let username: String = dialoguer::Input::new()
                    .with_prompt(t!("login.name"))
                    .interact_text()?;

                //User registration
                let res = rest_client::register_account(&email, &password, &username).await;
                match res {
                    Ok(r) => {
                        if r.status().is_success() {
                            //User register successful
                            println!("{}", t!("login.acc_created"));
                            return Ok(())

                        } else {
                            //Register failed either bad email or account already registered
                            println!(
                                "{}: {}",
                                t!("login.register_fail"),
                                r.json::<rest_client::Output>().await?.message
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
        let email: String = dialoguer::Input::new()
            .with_prompt(t!("login.user"))
            .interact_text()?;

        let password: String = dialoguer::Password::new()
            .with_prompt(t!("login.pass"))
            .interact()?;

        let res = rest_client::login_account(&email, &password).await;
        match res {
            Ok(r) => {
                if r.status().is_success() {
                    //User login successful
                    println!("{}", t!("login.success"));
                    return Ok(());

                } else {
                    //Incorrect credentials
                    println!(
                        "{}: {}",
                        t!("login.log_failed"),
                        r.json::<rest_client::Output>().await?.message
                    );
                }
            }
            Err(e) => {
                println!("{}. {}", e, t!("login.no_connection"));
            }
        }
    }
}

pub async fn setup() -> Result<(), Box<dyn Error>> {
    println!("{}", t!("setup_ini"));

    //Confirm selection loop
    loop {
        let online = dialoguer::Select::new()
            .with_prompt(t!("online.prompt"))
            .items(vec![t!("online.official"), t!("online.unofficial")])
            .interact()?;

        if online == 0 {
            let register = dialoguer::Select::new()
                .with_prompt(t!("login.registration"))
                .items(vec![t!("login.no_acc"), t!("login.acc")])
                .interact()?;

            if register == 0 {
                register_loop().await?;
                break;
            }
            else {
                login_loop().await?;
                break;
            }
        } else {
            if dialoguer::Confirm::new()
                .with_prompt(t!("online.confirm_prompt"))
                .interact()?
            {
                break;
            }
        }
    }

    println!("{}", t!("db_setup"));

    let created = db_client::create_tables();
    if let Err(e) = created {
        panic!("{}", t!("db_panic", error = e));
    }

    Ok(())
}
