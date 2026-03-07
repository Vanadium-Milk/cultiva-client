use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use common::db_client::Reading;
use common::rest_client::{Output, get_evaluation};
use common::state_handling::ActivationState;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::write;
use std::io;
use std::io::ErrorKind::Other;
use serde_json::json;

#[derive(Deserialize)]
struct SupervisionResponse {
    message: String,
    command: ActivationState,
    health: String,
}

pub(super) async fn evaluate(
    readings: Vec<Reading>,
    context: HashMap<String, String>,
    activation: ActivationState,
    image: Vec<u8>,
) -> Result<ActivationState, Box<dyn Error>> {
    let encoded = BASE64_STANDARD.encode(image);
    let eval = get_evaluation(readings, context, activation, encoded).await?;

    if eval.status().is_success() {
        let data = eval.json::<SupervisionResponse>().await?;

        let file = "/var/lib/cultiva/assessment.json";
        let content = json!({"health": data.health, "message": data.message });
        if let Err(e) = write(file, content.to_string()) {
            eprintln!("{}", t!("write_err", filename = file, error = e));
        };

        return Ok(data.command);
    }

    //Request was made but returned an error
    let error_msg = eval.json::<Output>().await?.message;
    Err(Box::new(io::Error::new(
        Other,
        t!("supervision.request_err", message = error_msg),
    )))
}
