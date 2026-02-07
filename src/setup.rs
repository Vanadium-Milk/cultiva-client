use crate::db_client;

pub fn setup() {
    println!("Setting up database...");

    let created = db_client::create_tables();
    if let Err(e) = created {
        panic!("Error creating the database, make sure to run with root permissions: {}", e);
    }
}