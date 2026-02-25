use rust_socketio::client::Client;
use rust_socketio::{Payload, RawClient};
use serde::Serialize;
use serde_json::{Value, json};
use std::env::var;
use std::fs::read_to_string;
use std::thread::sleep;
use std::time::Duration;

//Not bool because There'll be more statuses probably in the future
#[derive(Serialize)]
pub(super) enum ResponseStatus {
    Success,
    Failed,
}

fn payload_to_string(payload: Payload) -> String {
    if let Payload::Text(content) = payload {
        let mut all = "".to_string();

        for t in content {
            all += &*(t.to_string() + " ");
        }

        return all;
    }
    "No readable content".to_string()
}

pub(super) fn on_success(payload: Payload, _socket: RawClient) {
    println!(
        "{}",
        t!("socket_io.success", message = payload_to_string(payload))
    );
}

pub(super) fn on_failure(payload: Payload, _socket: RawClient) {
    eprintln!(
        "{}",
        t!("socket_io.failed", message = payload_to_string(payload))
    );
}

//Authenticate with JWT stored by systemd-creds
pub(super) fn authenticate_connection(payload: Payload, client: RawClient) {
    //NOTE to skip launching the app as a systemd service in development builds, create an environment
    //variable called JWT with a path pointing to a plaintext file containing your token
    println!(
        "{}",
        t!("socket_io.message", message = payload_to_string(payload))
    );

    let cred_dir = var("JWT");
    if let Ok(cred) = cred_dir
        && let Ok(token) = read_to_string(cred)
    {
        let auth = "Bearer ".to_owned() + &token;

        //10 attempts at reconnection
        for _i in 0..10 {
            match client.emit("authenticate", auth.trim_end()) {
                Ok(_) => {
                    println!("{}", t!("socket_io.auth.success"));
                    return;
                }
                Err(e) => {
                    eprint!("{}. {}", t!("socket_io.auth.error", error = e), t!("retry"));
                    sleep(Duration::from_secs(10));
                }
            }
        }
        eprintln!("{}", t!("socket_io.auth.failed"));
        if let Err(e) = client.disconnect() {
            eprintln!("{}", t!("socket_io.disconnect_err", error = e));
        }
    } else {
        eprintln!("{}", t!("socket_io.auth.read_err"));
        if let Err(e) = client.disconnect() {
            eprintln!("{}", t!("socket_io.disconnect_err", error = e));
        }
    }
}

pub(super) fn send_data(socket: &RawClient, payload: Value) {
    match socket.emit("response", payload) {
        Ok(_) => {
            println!("{}", t!("query.sent"));
        }
        Err(e) => {
            eprintln!("{}", t!("query.send_error", error = e));
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

pub(super) fn test_connection(client: Client) {
    loop {
        sleep(Duration::from_mins(3));
        if let Err(e) = client.emit("test_connection", "ping") {
            eprintln!("{}", t!("socket_io.lost", error = e));
        }
    }
}
