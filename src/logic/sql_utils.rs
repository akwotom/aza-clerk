/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module is concerned with the certain SQL utilities.
*/

use crate::logic::db::DbHandle;

/// This method creates the SQL stored procedures in DB.
pub async fn run_multiple_sql(statements: &[&str], db: &DbHandle) -> Result<(), sqlx::Error> {
    let connection = db.connect().await?;

    let mut threads = tokio::task::JoinSet::<Result<(), sqlx::Error>>::new();

    for stmt in statements {
        let statement = stmt.to_string();
        let conn = connection.clone();
        threads.spawn(async move {
            let res = sqlx::raw_sql(sqlx::AssertSqlSafe(statement))
                .execute(&conn)
                .await;
            res.map(|_| ())
        });
    }

    let results = threads.join_all().await;

    for result in results {
        // If there's just as much as a single error with the procedures, then let's alert the caller.
        if let Result::Err(e) = result {
            return Result::Err(e);
        }
    }

    Result::Ok(())
}
