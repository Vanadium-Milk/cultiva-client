use crate::credentials::get_jwt;
use crate::db_client::Reading;
use crate::state_handling::ActivationState;
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::env::var;
use std::error::Error;
use std::io;
use std::io::ErrorKind::HostUnreachable;

#[derive(Deserialize)]
pub struct Output {
    #[allow(non_snake_case)]
    pub statusCode: i32,
    pub message: String,
}

#[derive(Deserialize)]
pub struct Auth {
    pub token: String,
}

pub async fn register_account(
    email: &str,
    password: &str,
    username: &str,
) -> Result<Response, Box<dyn Error>> {
    let url = format!("{}/users", var("REST_URL")?);

    let user_register =
        HashMap::from([("email", email), ("password", password), ("name", username)]);
    let res = Client::new().post(url).json(&user_register).send().await?;

    Ok(res)
}

pub async fn login_account(email: &str, password: &str) -> Result<Response, Box<dyn Error>> {
    let url = format!("{}/users/login", var("REST_URL")?);

    let user_login = HashMap::from([("email", email), ("password", password)]);
    let res = Client::new().post(url).json(&user_login).send().await?;

    Ok(res)
}

//Error cannot be boxed due to usage in threaded async
pub async fn get_evaluation(
    readings: Vec<Reading>,
    context: HashMap<String, String>,
    activation: ActivationState,
    image: String,
) -> Result<Response, io::Error> {
    let url = format!(
        "{}/supervision",
        var("REST_URL").unwrap_or("api.proyectocultiva.org".to_string())
    );

    //json! macro includes None values as null, I converted it to HashMap first to remove them
    let clean_act: HashMap<String, bool> = activation.into();
    let content = json!({
        "readings": readings,
        "context": context,
        "activation": clean_act,
        "image": image
    });

    let response = Client::new()
        .post(url)
        .json(&content)
        .bearer_auth(get_jwt()?)
        .send()
        .await
        .unwrap_or(Err(io::Error::new(HostUnreachable, t!("http.error")))?);
    Ok(response)
}
