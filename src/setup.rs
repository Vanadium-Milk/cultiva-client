use crate::db_client;
use crate::rest_client;
use dialoguer;
use std::error::Error;
use crate::rest_client::login_account;

i18n!();

pub async fn setup() -> Result<(), Box<dyn Error>> {
    println!("{}", t!("setup_ini"));

    //Confirm selection loop
    while true {
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
                println!("{}", t!("login.disclaimer"));
            }
            let email: String = dialoguer::Input::new()
                .with_prompt(t!("login.user"))
                .interact_text()?;

            //Password re-attempt loop
            while true {
                let password: String = dialoguer::Password::new()
                    .with_prompt(t!("login.pass"))
                    .interact()?;

                if register == 0 {
                    //Confirm password
                    if password
                        == dialoguer::Password::new()
                            .with_prompt(t!("login.pass_confirm"))
                            .interact()?
                    {
                        let username: String = dialoguer::Input::new()
                            .with_prompt(t!("login.name"))
                            .interact_text()?;
                        let res = rest_client::register_account(&email, &password, &username).await;
                        if res.is_err() {
                            println!("{}", res.unwrap_err());
                        }
                        else {
                            break;
                        }
                    } else {
                        println!("{}", t!("login.pass_match_fail"));
                    }
                }
                else {
                    if login_account(&email, &password).await.is_err() {
                        println!("{}", t!("login.failed"));
                    }
                    else{
                        break;
                    }
                }
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
