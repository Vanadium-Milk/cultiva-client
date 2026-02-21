use crate::db_client::{
    Reading, create_tables, delete_readings, get_last_reading, get_readings, insert_reading,
};
use rusqlite::Error as dbError;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_database() -> Result<(), dbError> {
    //Table creation ----------------------------------------------------------------------------
    println!("Creating database tables...");
    create_tables()?;

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

    insert_reading(test_read)?;

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
    insert_reading(test_read)?;

    let last = get_last_reading()?;
    let all = get_readings(2)?;

    println!("last: {:?}", last);
    println!("all: {:?}", all);

    //Cleanup ----------------------------------------------------------------------------------
    println!("Removing inserted rows...");
    delete_readings()?;

    Ok(())
}
