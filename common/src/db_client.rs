use rusqlite::{Connection, Error, Row};
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Reading {
    pub timestamp: Option<String>,
    pub temperature: Option<f32>,
    pub air_humidity: Option<f32>,
    pub soil_humidity: Option<f32>,
    pub luminosity: Option<f32>,
    pub air_quality: Option<f32>,
    pub ph: Option<f32>,
}

impl Reading {
    pub fn new() -> Self {
        Default::default()
    }
}

fn get_connection() -> rusqlite::Result<Connection, Error> {
    let path = "/var/lib/cultiva/readings.db3";
    let db = Connection::open(path)?;

    Ok(db)
}

fn parse_reading() -> fn(&Row) -> Result<Reading, Error> {
    |row| {
        Ok(Reading {
            timestamp: row.get(0)?,
            temperature: row.get(1)?,
            air_humidity: row.get(2)?,
            soil_humidity: row.get(3)?,
            luminosity: row.get(4)?,
            air_quality: row.get(5)?,
            ph: row.get(6)?,
        })
    }
}

// Public functions --------------------------------------------------------------------------------
pub fn create_tables() -> Result<(), Error> {
    let connection = get_connection()?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS readings (
            time_stamp  TIMESTAMP PRIMARY KEY DEFAULT CURRENT_TIMESTAMP,
            temperature REAL,
            air_hum     REAL UNSIGNED,
            soil_hum    REAL UNSIGNED,
            light       REAL UNSIGNED,
            air_quality REAL UNSIGNED,
            ph          REAL UNSIGNED
            )",
        (),
    )?;

    Ok(())
}

pub fn insert_reading(values: Reading) -> Result<(), Error> {
    let connection = get_connection()?;
    connection.execute(
        "INSERT INTO readings (temperature, air_hum, soil_hum, light, air_quality, ph) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (values.temperature, values.air_humidity, values.soil_humidity, values.luminosity, values.air_quality, values.ph),
    )?;

    Ok(())
}

pub fn get_last_reading() -> Result<Reading, Error> {
    let connection = get_connection()?;

    let res = connection.query_one(
        "SELECT * FROM readings ORDER BY time_stamp DESC LIMIT 1 ",
        (),
        parse_reading(),
    )?;
    Ok(res)
}

pub fn get_readings(limit: u64) -> Result<Vec<Reading>, Error> {
    if limit == 1 {
        //I suppose this is faster
        return Ok(vec![get_last_reading()?]);
    }
    let connection = get_connection()?;

    let mut stmt = connection.prepare(
        format!(
            "SELECT * FROM readings ORDER BY time_stamp DESC LIMIT {}",
            limit
        )
        .as_str(),
    )?;
    let res = stmt.query_map([], parse_reading())?;
    let data: Result<Vec<Reading>, Error> = res.collect();

    data
}

//Only for debug, remove all records
pub fn delete_readings() -> Result<(), Error> {
    let connection = get_connection()?;
    connection.execute("DELETE FROM readings", ())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db_client::{
        Reading, create_tables, get_last_reading, get_readings, insert_reading,
    };
    use rusqlite::Error;
    use std::thread::sleep;
    use std::time::Duration;

    fn create() -> Result<(), Error> {
        //Table creation ----------------------------------------------------------------------------
        println!("Creating database tables...");
        create_tables()?;

        Ok(())
    }
    fn insert() -> Result<(), Error> {
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

        Ok(())
    }

    fn select() -> Result<(), Error> {
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

        Ok(())
    }
}
