mod capture;
mod serial;
mod socket_io;
pub mod supervision;

use crate::service::capture::{get_image_buffer, poll_cam};
use crate::service::serial::Modes::{Active, Auto};
use crate::service::serial::{BoardControl, register_data};
use crate::service::socket_io::{
    authenticate_connection, on_failure, on_success, report_result, send_data, test_connection,
};
use crate::service::supervision::evaluate;
use chrono::Utc;
use common::context::{get_context, set_context};
use common::db_client::get_readings;
use common::settings::load_conf;
use common::state_handling::ActivationState;
use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde_json::json;
use std::env::var;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};

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

fn on_capture(payload: Payload, client: RawClient) {
    if let Payload::Text(text) = &payload
        && !text.is_empty()
        && let Some(response_id) = text[0].as_str()
    {
        match get_image_buffer() {
            Ok(buffer) => send_data(
                &client,
                json!({
                "id": response_id,
                "data": {
                        "buffer": buffer
                    },
                "success": true
                }),
            ),
            Err(e) => report_result(client, response_id, false, &e.to_string()),
        }
    } else {
        eprintln!("{}: {:?}", t!("socket_io.payload_invalid"), payload);
    }
}

async fn supervise(board: Arc<Mutex<BoardControl>>) {
    println!("{}", t!("supervision.start"));
    if let Ok(readings) = get_readings(20)
        && let Ok(context) = get_context()
        && let Ok(image) = get_image_buffer()
    {
        //Prevent mutex being locked while awaiting by wrapping it in this block
        let eval = match board.lock() {
            Ok(locked) => evaluate(readings, context, locked.state, image),
            Err(e) => {
                eprintln!("{}, {}", t!("serial.lock_error"), e);
                return;
            }
        };

        match eval.await {
            Ok(evaluation) => {
                let Ok(mut locked) = board.lock() else {
                    eprintln!("{}", t!("serial.lock_error"));
                    return;
                };
                if let Err(e) = locked.set_activation(evaluation) {
                    eprintln!("{}", t!("serial.command.error", error = e));
                }
            }
            Err(e) => {
                eprintln!("{}", t!("supervision.net_error", error = e));
            }
        }
    } else {
        eprintln!("{}", t!("supervision.no_data"));
    }
}

fn initiate_socket(board: Option<Arc<Mutex<BoardControl>>>) {
    let comm_arc = board.clone();
    let act_arc = board.clone();

    //Callback to pass the port value to the command handling function
    let command_callback = move |payload: Payload, socket: RawClient| {
        if let Payload::Text(text) = &payload
            && text.len() >= 3
            && let Some(response_id) = text[0].as_str()
            && let Some(mode) = text[1].as_str()
            && let Ok(command) = serde_json::from_value::<ActivationState>(text[2].clone())
        {
            if let Some(board) = &comm_arc
                && let Ok(mut locked) = board.lock()
            {
                let result = match mode {
                    "auto" => locked.set_auto_modes(command),
                    _ => locked.set_activation(command),
                };
                match result {
                    Ok(_) => {
                        report_result(socket, response_id, true, "Command performed successfully");
                    }
                    Err(e) => report_result(socket, response_id, false, &e.to_string()),
                }
            } else {
                report_result(
                    socket,
                    response_id,
                    false,
                    t!("serial.unavailable").as_ref(),
                );
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
            if let Some(board) = &act_arc
                && let Ok(locked) = board.lock()
            {
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
            } else {
                report_result(
                    client,
                    response_id,
                    false,
                    t!("serial.unavailable").as_ref(),
                );
            }
        } else {
            eprintln!("{}: {:?}", t!("socket_io.payload_invalid"), payload);
        }
    };

    println!("{}", t!("socket_io.connecting"));
    //Initiate a socket.io connection
    match ClientBuilder::new(var("REST_URL").unwrap_or("api.proyectocultiva.org".to_string()))
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
        .connect()
    {
        Ok(connection) => {
            test_connection(connection);
            return;
        }
        Err(e) => eprintln!("{}", t!("socket_io.error", error = e)),
    }
    eprintln!("{}", t!("socket_io.disable"));
}

pub(super) async fn start_tasks() -> Result<(), Box<dyn Error>> {
    println!("{}", t!("config.load"));
    let config = load_conf()?;

    println!("{}", t!("serial.initializing", port = config.board.port));
    let board_arc = match serialport::new(config.board.port, 9600)
        .timeout(Duration::from_secs(5))
        .open()
    {
        Ok(port) => Some(Arc::new(Mutex::new(BoardControl::new(port)))),
        Err(e) => {
            eprintln!("{}", t!("serial.init_error", error = e));
            None
        }
    };

    if let Some(board) = board_arc.clone() {
        println!("{}", t!("sched.start"));
        let sched = JobScheduler::new().await?;

        let reg_board = board.clone();
        spawn(move || register_data(reg_board));

        sched
            .add(Job::new_async_tz("0 0 12 * * *", Utc, move |_, _| {
                let sup_board = board.clone();
                Box::pin(async {
                    supervise(sup_board).await;
                })
            })?)
            .await?;

        sched.start().await?;
    }

    spawn(move || initiate_socket(board_arc.clone()));

    poll_cam();

    Ok(())
}

#[tokio::test]
async fn test_supervision() {
    //Change it accordingly
    let device = "/dev/ttyUSB0";
    let port = serialport::new(device, 9600)
        .timeout(Duration::from_secs(5))
        .open()
        .unwrap();

    supervise(Arc::new(Mutex::new(BoardControl::new(port)))).await;
}
