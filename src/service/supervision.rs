use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use common::db_client::Reading;
use common::rest_client::{Output, get_evaluation};
use common::state_handling::ActivationState;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{read_to_string, write};
use std::io;
use std::io::ErrorKind::Other;

#[derive(Deserialize, Serialize)]
pub(super) struct Threshold {
    pub(super) min: f32,
    pub(super) max: f32,
}

#[derive(Deserialize, Serialize)]
pub(super) struct VariableRange {
    pub(super) temperature: Threshold,
    pub(super) soil_humidity: Threshold,
    pub(super) air_humidity: Threshold,
    pub(super) luminosity: Threshold,
    pub(super) co2: Threshold,
}

#[derive(Deserialize)]
struct SupervisionResponse {
    message: String,
    command: ActivationState,
    health: String,
    advice: Vec<String>,
    ranges: VariableRange,
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
        let content =
            json!({"health": data.health, "message": data.message, "advice": data.advice });
        if let Err(e) = write(file, content.to_string()) {
            eprintln!("{}", t!("write_err", filename = file, error = e));
        };

        let ranges = "/var/lib/cultiva/ranges.toml";
        if let Ok(content) = toml::to_string(&data.ranges)
            && let Err(e) = write(ranges, content)
        {
            eprintln!("{}", t!("write_err", filename = ranges, error = e));
        }

        return Ok(data.command);
    }

    //Request was made but returned an error
    let error_msg = eval.json::<Output>().await?.message;
    Err(Box::new(io::Error::new(
        Other,
        t!("supervision.request_err", message = error_msg),
    )))
}

pub(super) fn get_assessment() -> Result<Value, Box<dyn Error>> {
    let content = read_to_string("/var/lib/cultiva/assessment.json")?;

    Ok(serde_json::from_str(&content)?)
}

pub(super) fn get_ranges() -> Result<VariableRange, Box<dyn Error>> {
    let content = read_to_string("/var/lib/cultiva/ranges.toml")?;
    let serialize = toml::from_str::<VariableRange>(&content)?;

    Ok(serialize)
}
