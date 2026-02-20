use crate::db_client::Reading;
use rust_socketio::client::Client;
use rust_socketio::{Payload, RawClient};
use serde::Serialize;
use serde_json::json;
use std::env::var;
use std::error::Error;
use std::fs::read_to_string;
use std::time::Duration;

#[derive(Serialize)]
struct Response {
    id: String,
    data: Vec<Reading>,
}

fn display_response(payload: Payload, _socket: RawClient) {
    dbg!(payload);
}

//Authenticate with JWT stored by systemd-creds
pub(super) fn authenticate_connection(socket: &Client) -> Result<(), Box<dyn Error>> {
    //NOTE to skip launching the app as a systemd service in development builds, create an environment
    //variable called JWT with a path pointing to a plaintext file containing your token
    let cred = var("JWT")?;
    let token = "Bearer ".to_owned() + &*read_to_string(cred)?;
    socket.emit_with_ack(
        "authenticate",
        token.trim_end(),
        Duration::from_secs(5),
        display_response,
    )?;
    Ok(())
}

pub(super) fn send_readings(
    socket: &RawClient,
    response_id: &str,
    data: Vec<Reading>,
) -> Result<(), Box<dyn Error>> {
    let send = Response {
        id: response_id.to_owned(),
        data,
    };
    let payload = json!(send);
    socket.emit("response", payload)?;
    Ok(())
}
