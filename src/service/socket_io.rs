use rust_socketio::client::Client;
use rust_socketio::{Payload, RawClient};
use serde::Serialize;
use serde_json::{Value, json};
use std::env::var;
use std::error::Error;
use std::fs::read_to_string;

//Not bool because There'll be more statuses probably in the future
#[derive(Serialize)]
pub(super) enum ResponseStatus {
    Success,
    Failed,
}

pub(super) fn on_success(payload: Payload, _socket: RawClient) {
    println!("{}: {:?}", t!("socket_io.success"), payload);
}

pub(super) fn on_failure(payload: Payload, _socket: RawClient) {
    eprintln!("{}: {:?}", t!("socket_io.failed"), payload);
}

//Authenticate with JWT stored by systemd-creds
pub(super) fn authenticate_connection(socket: &Client) -> Result<(), Box<dyn Error>> {
    //NOTE to skip launching the app as a systemd service in development builds, create an environment
    //variable called JWT with a path pointing to a plaintext file containing your token
    let cred = var("JWT")?;
    let token = "Bearer ".to_owned() + &*read_to_string(cred)?;
    socket.emit("authenticate", token.trim_end())?;
    Ok(())
}

pub(super) fn send_data(socket: &RawClient, payload: Value) {
    match socket.emit("response", payload) {
        Ok(_) => {
            println!("{}", t!("socket_io.sent"));
        }
        Err(e) => {
            eprintln!("{}", t!("socket_io.send_error", error = e));
        }
    }
}

pub(super) fn report_result(
    socket: RawClient,
    response_id: &str,
    result: ResponseStatus,
    message: &str,
) {
    let res = socket.emit(
        "response",
        json!({
            "id": response_id,
            "data": {
                "status": result,
                "message": message
            }
        }),
    );

    match res {
        Ok(_) => {
            println!("{}", t!("socket_io.report.success"));
        }
        Err(e) => {
            eprintln!("{}", t!("socket_io.report.failure", error = e));
        }
    }
}
