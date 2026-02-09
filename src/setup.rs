use std::error::Error;
use crate::db_client;
use dialoguer;

i18n!();

pub fn setup() -> Result<(), Box<dyn Error>> {
    println!("{}", t!("setup_ini"));

    let online = dialoguer::Select::new()
        .with_prompt(t!("online_prompt"))
        .items(vec![t!("online"), t!("offline")])
        .interact()?;

    if online == 0 {
        let register = dialoguer::Select::new()
            .with_prompt(t!("login.registration"))
            .items(vec![t!("login.no_acc"), t!("login.acc")])
            .interact()?;

        println!("{}", t!("login.disclaimer"));
        let username: String = dialoguer::Input::new()
            .with_prompt(t!("login.user"))
            .interact_text()?;

        let password: String = dialoguer::Password::new()
            .with_prompt(t!("login.pass"))
            .interact()?;
    }


    println!("{}", t!("db_setup"));

    let created = db_client::create_tables();
    if let Err(e) = created {
        panic!("{}", t!("db_panic", error = e));
    }

    Ok(())
}