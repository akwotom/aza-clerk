/*
    Copyright 2026 Son of Binary
    The aza-clerk project
    This module serves as the entry-point of the microservice.

*/

use crate::logic::foreign_exchange::ForeignExchange;

mod http;
mod logic;
mod models;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_url = get_env(
        "MYSQL_DATABASE_URL",
        "Could not connect to the database, because",
    );

    let fx_api_key = get_env(
        "FX_API_KEY",
        "Currency conversion API key is missing, because",
    );

    let db = logic::db::DbHandle::new(db_url);

    logic::init(&db).await.unwrap();

    models::init(&db).await.unwrap();

    let port_env = "PORT";

    let port = std::env::var(port_env)
        .map_err(|_| {
            format!("Could not start the HTTP server because the '{port_env}' variable is missing.")
        })
        .unwrap();

    let port = port.parse().map_err(|_| {
        format!(
            "Could not start HTTP server because of an invalid value for the '{port_env}' variable."
        )
    }).unwrap();

    http::server::create_server(port, db, ForeignExchange::new(fx_api_key)).await;
}

fn get_env(key_name: &str, err_template: &str) -> String {
    std::env::var(key_name)
        .map_err(|_| {
            format!(
                "{} '{key_name}' environment variable was not set.\n",
                err_template
            )
        })
        .unwrap()
}
