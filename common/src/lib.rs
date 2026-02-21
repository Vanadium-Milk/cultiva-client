#[macro_use]
extern crate rust_i18n;
i18n!();

pub mod db_client;
pub mod rest_client;
pub mod settings;

#[cfg(test)]
mod tests;
