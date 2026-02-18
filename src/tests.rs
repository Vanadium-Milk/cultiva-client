use crate::db_client::Reading;
use crate::rest_client;
use reqwest::Error as httpError;
use rusqlite::Error as dbError;
use std::env::var;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_database() -> Result<(), dbError> {
    //Table creation ----------------------------------------------------------------------------
    println!("Creating database tables...");
    crate::db_client::create_tables()?;

    //Data insertion ---------------------------------------------------------------------------
    println!("Testing data insertion");

    let test_read = Reading {
        timestamp: None,
        temperature: Some(32.0),
        air_humidity: Some(14.5),
        soil_humidity: Some(154.0),
        luminosity: Some(100.0),
        air_quality: Some(100.0),
        ph: Some(8.5),
    };

    crate::db_client::insert_reading(test_read)?;

    //Data querying ----------------------------------------------------------------------------
    println!("Testing data querying");
    let test_read = Reading {
        timestamp: None,
        temperature: Some(53.0),
        air_humidity: Some(17.5),
        soil_humidity: Some(54.0),
        luminosity: Some(70.0),
        air_quality: Some(680.0),
        ph: Some(7.5),
    };
    sleep(Duration::from_secs(1));
    crate::db_client::insert_reading(test_read)?;

    let last = crate::db_client::get_last_reading()?;
    let all = crate::db_client::get_readings(2)?;

    println!("last: {:?}", last);
    println!("all: {:?}", all);

    //Cleanup ----------------------------------------------------------------------------------
    println!("Removing inserted rows...");
    crate::db_client::delete_readings()?;

    Ok(())
}

#[tokio::test]
async fn test_rest_client() -> Result<(), httpError> {
    println!("Current server: {:?}", var("REST_URL"));

    //Register -------------------------------------------------------------------------------------
    println!("Registering account");
    let response = rest_client::register_account("test@test.com", "admin123", "test").await?;
    println!(
        "Server response: {:?}, {:?}",
        response.status(),
        response.text().await?
    );

    //Login ----------------------------------------------------------------------------------------
    println!("Login into account");
    let response = rest_client::login_account("test@test.com", "admin123").await?;
    println!(
        "Server response: {:?}, {:?}",
        response.status(),
        response.text().await?
    );

    Ok(())
}
