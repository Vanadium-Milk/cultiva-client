#[macro_use]
extern crate rust_i18n;
i18n!(fallback = "en");

pub mod context;
pub mod credentials;
pub mod db_client;
pub mod locales;
pub mod rest_client;
pub mod settings;
pub mod state_handling;
