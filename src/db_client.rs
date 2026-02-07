use rusqlite::{Connection, Error, Row};

#[derive(Debug)]
pub(crate) struct Reading {
    pub(crate) timestamp: Option<String>,
    pub(crate) temperature: Option<f32>,
    pub(crate) air_humidity: Option<f32>,
    pub(crate) soil_humidity: Option<f32>,
    pub(crate) luminosity: Option<f32>,
    pub(crate) air_quality: Option<f32>,
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
        })
    }
}

// Public functions --------------------------------------------------------------------------------
pub(crate) fn create_tables() -> Result<(), Error> {
    let connection = get_connection()?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS readings (
            time_stamp  TIMESTAMP PRIMARY KEY DEFAULT CURRENT_TIMESTAMP,
            temperature REAL,
            air_hum     REAL UNSIGNED,
            soil_hum    REAL UNSIGNED,
            light       REAL UNSIGNED,
            air_quality REAL UNSIGNED
            )",
        (),
    )?;

    Ok(())
}

pub(crate) fn insert_reading(values: Reading) -> Result<(), Error> {
    let connection = get_connection()?;
    connection.execute(
        "INSERT INTO readings (temperature, air_hum, soil_hum, light, air_quality) VALUES (?1, ?2, ?3, ?4, ?5)",
        (values.temperature, values.air_humidity, values.soil_humidity, values.luminosity, values.air_quality)
    )?;

    Ok(())
}

pub(crate) fn get_last_reading() -> Result<Reading, Error> {
    let connection = get_connection()?;

    let res = connection.query_one(
        &"SELECT * FROM readings ORDER BY time_stamp DESC LIMIT 1 ",
        (),
        parse_reading(),
    )?;
    Ok(res)
}

pub(crate) fn get_readings(limit: i32) -> Result<Vec<Reading>, Error> {
    let connection = get_connection()?;

    let mut stmt = connection
        .prepare(format!("SELECT * FROM readings ORDER BY time_stamp DESC LIMIT {}", limit).as_str())?;
    let res = stmt.query_map([], parse_reading())?;
    let data: Result<Vec<Reading>, Error> = res.collect();

    data
}

//Only for debug, remove all records
pub(crate) fn delete_readings() -> Result<(), Error> {
    let connection = get_connection()?;
    connection.execute("DELETE FROM readings", ())?;

    Ok(())
}