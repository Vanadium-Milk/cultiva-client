use reqwest::{Client, Response};
use serde::Deserialize;
use std::collections::HashMap;
use std::env::var;

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
) -> Result<Response, reqwest::Error> {
    let url = format!("{}/users", var("REST_URL").expect(""));

    let client = Client::new();

    let mut user_register = HashMap::new();
    user_register.insert("email", email);
    user_register.insert("password", password);
    user_register.insert("name", username);

    let res = client.post(url).json(&user_register).send().await?;

    Ok(res)
}

pub async fn login_account(email: &str, password: &str) -> Result<Response, reqwest::Error> {
    let url = format!("{}/users/login", var("REST_URL").expect(""));

    let client = Client::new();
    let mut user_login = HashMap::new();
    user_login.insert("email", email);
    user_login.insert("password", password);

    let res = client.post(url).json(&user_login).send().await?;

    Ok(res)
}
