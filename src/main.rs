/*
    Copyright 2026 Son of Binary
    The aza-clerk project
    This module serves as the entry-point of the microservice.

*/

mod http;
mod logic;
mod models;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_env_name = "MYSQL_DATABASE_URL";
    let db_url = std::env::var(db_env_name)
        .map_err(|_| {
            format!("Could not connect to the database, because {db_env_name} was not set.\n")
        })
        .unwrap();
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

    http::server::create_server(port, db).await;

    println!("Okay. All good!");
}
