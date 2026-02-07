#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use std::time::Duration;
    use rusqlite::Error;
    use crate::db_client::Reading;
    #[test]
    fn test_database () -> Result<(), Error> {
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
            air_quality: Some(100.0)
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
            air_quality: Some(680.0)
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
}