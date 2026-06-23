/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains features that directly concern the operation of the database.

*/

#[derive(Clone)]
pub struct DbHandle {
    pool: sqlx::mysql::MySqlPoolOptions,
    db_url: String,
}

impl DbHandle {
    /// This method does the actual work of connecting to the DB, and returning a working connection.
    pub async fn connect(&self) -> Result<sqlx::Pool<sqlx::MySql>, sqlx::Error> {
        let connection = self.pool.clone().connect(&self.db_url).await?;
        Result::Ok(connection)
    }

    pub fn new(db_url: String) -> Self {
        let pool = sqlx::mysql::MySqlPoolOptions::new().max_connections(5);

        Self { db_url, pool }
    }
}
