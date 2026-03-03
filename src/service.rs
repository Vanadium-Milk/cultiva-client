mod capture;
mod serial;
mod socket_io;

use crate::service::capture::{poll_cam, save_frame};
use crate::service::serial::Modes::{Active, Auto};
use crate::service::serial::{BoardControl, register_data};
use crate::service::socket_io::{
    authenticate_connection, on_failure, on_success, report_result, send_data, test_connection,
};
use common::context::{get_context, set_context};
use common::db_client::get_readings;
use common::settings::load_conf;
use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde_json::json;
use std::env::var;
use std::error::Error;
use std::fs::{read, read_dir};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, SystemTime};

fn on_query(payload: Payload, raw_client: RawClient) {
    if let Payload::Text(text) = &payload
        && text.len() >= 2
        && let Some(response_id) = text[0].as_str()
        && let Some(amount) = text[1].as_u64()
    {
        match get_readings(amount) {
            Ok(readings) => send_data(
                &raw_client,
                json!({
                    "id": response_id,
                    "data": readings,
                    "success": true
                }),
            ),
            Err(e) => {
                report_result(raw_client, response_id, false, &e.to_string());
                eprintln!("{}", t!("query.retrieve_error", error = e));
            }
        }
    } else {
        eprintln!("{}: {:?}", t!("socket_io.payload_invalid"), payload);
    }
}

fn on_context(payload: Payload, raw_client: RawClient) {
    if let Payload::Text(text) = &payload
        && text.len() >= 2
        && let Some(response_id) = text[0].as_str()
    {
        if let Some(_flag) = text[1].as_str() {
            match get_context() {
                Ok(context) => send_data(
                    &raw_client,
                    json!({
                        "id": response_id,
                        "data": context,
                        "success": true
                    }),
                ),
                Err(e) => {
                    eprintln!("{}", t!("context.load_err", error = e));
                    report_result(raw_client, response_id, false, &e.to_string());
                }
            }
        } else {
            let status;
            let message;
            match serde_json::from_value(text[1].clone()) {
                Ok(context) => {
                    if let Err(e) = set_context(context) {
                        status = false;
                        message = e.to_string();
                        eprintln!("{}", t!("context.save_err", error = e));
                    } else {
                        status = true;
                        message = "Success saving context information".to_string();
                    }
                }
                Err(e) => {
                    status = false;
                    message = e.to_string();
                    eprintln!("{}", t!("context.parse_err", error = e));
                }
            }
            report_result(raw_client, response_id, status, &message)
        }
    } else {
        eprintln!("{}: {:?}", t!("socket_io.payload_invalid"), payload);
    }
}

pub(super) fn on_capture(payload: Payload, client: RawClient) {
    if let Payload::Text(text) = &payload
        && !text.is_empty()
        && let Some(response_id) = text[0].as_str()
    {
        let mut buffer = None;
        //Save frame when image is requested
        match save_frame() {
            Ok(name) => {
                //If capture succeeds simply return the image
                if let Ok(img) = read(format!("var/lib/cultiva/captures/{}.jpg", name)) {
                    buffer = Some(img);
                }
            }
            Err(e) => {
                //If capture fails simply use the most recent one instead
                eprintln!("{}", t!("capture.failed", error = e));

                //This monstrosity returns the last created file
                if let Ok(paths) = read_dir("/var/lib/cultiva/captures/")
                    && let Some(last) = paths.max_by_key(|entry| {
                        if let Ok(val) = entry
                            && let Ok(meta) = val.metadata()
                            && let Ok(time) = meta.created()
                        {
                            time
                        } else {
                            SystemTime::UNIX_EPOCH
                        }
                    })
                    && let Ok(entry) = last
                    && let Ok(img) = read(entry.path())
                {
                    buffer = Some(img);
                }
            }
        }

        if let Some(image_data) = buffer {
            send_data(
                &client,
                json!({
                "id": response_id,
                "data": {
                        "buffer": image_data
                    },
                "success": true
                }),
            );
        } else {
            eprintln!("capture.load_err");
            report_result(client, response_id, false, &t!("capture.load_err"));
        }
    }
}

pub(super) fn start_tasks() -> Result<(), Box<dyn Error>> {
    println!("{}", t!("config.load"));
    let config = load_conf()?;

    let board = Arc::new(Mutex::new(BoardControl::new(
        serialport::new(config.board.port, 9600)
            .timeout(Duration::from_secs(5))
            .open()?,
    )));

    //command and activation arc of the arduino board struct
    let act_arc = Arc::clone(&board);
    let comm_arc = Arc::clone(&board);

    //Callback to pass the port value to the command handling function
    let command_callback = move |payload: Payload, socket: RawClient| {
        if let Payload::Text(text) = &payload
            && text.len() >= 3
            && let Some(response_id) = text[0].as_str()
            && let Some(mode) = text[1].as_str()
        {
            match comm_arc.lock() {
                Ok(mut locked) => {
                    let result = match mode {
                        "auto" => locked.set_auto_modes(&text[2]),
                        _ => locked.set_activation(&text[2]),
                    };
                    match result {
                        Ok(_) => {
                            report_result(
                                socket,
                                response_id,
                                true,
                                "Command performed successfully",
                            );
                        }
                        Err(e) => report_result(socket, response_id, false, &e.to_string()),
                    }
                }
                Err(e) => {
                    eprintln!("{}", t!("serial.lock_error", error = e));
                }
            }
        } else {
            eprintln!("{}: {:?}", t!("socket_io.payload_invalid"), payload);
        }
    };

    let activation_callback = move |payload: Payload, client: RawClient| {
        if let Payload::Text(text) = &payload
            && text.len() >= 2
            && let Some(response_id) = text[0].as_str()
            && let Some(mode) = text[1].as_str()
        {
            match act_arc.lock() {
                Ok(mut locked) => {
                    let info = match mode {
                        "auto" => Auto,
                        _ => Active,
                    };
                    send_data(
                        &client,
                        json!({
                            "id": response_id,
                            "data": locked.get_activation(info),
                            "success": true
                        }),
                    )
                }
                Err(e) => {
                    eprintln!("{}", t!("serial.lock_error", error = e));
                    report_result(client, response_id, false, &e.to_string());
                }
            }
        } else {
            eprintln!("{}: {:?}", t!("socket_io.payload_invalid"), payload);
        }
    };

    println!("{}", t!("socket_io.connecting"));
    //Initiate a socket.io connection
    let conn = ClientBuilder::new(var("REST_URL")?)
        .on("command", command_callback)
        .on("query", on_query)
        .on("activation", activation_callback)
        .on("success", on_success)
        .on("error", on_failure)
        .on("authenticate", authenticate_connection)
        .on("capture", on_capture)
        .on("context", on_context)
        .reconnect(true)
        .reconnect_on_disconnect(true)
        .connect()?;

    //Added delay to ensure serial connection is ready
    sleep(Duration::from_secs(5));

    spawn(|| test_connection(conn));
    spawn(poll_cam);
    register_data(board);

    Ok(())
}
