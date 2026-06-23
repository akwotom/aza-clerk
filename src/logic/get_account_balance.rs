/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    The module contains the logic for retrieving the account balance of a user
*/

use crate::logic::db::DbHandle;

/// This method returns the balance of the
pub async fn by_account_id(
    account_id: String,
    db: DbHandle,
) -> Result<Option<crate::models::AccountBalance>, sqlx::Error> {
    let conn = db.connect().await.unwrap();

    let item = sqlx::query_as::<_, crate::models::AccountBalance>(
        "
            SELECT 
                currency AS amount_currency,
                (
                    CAST( COALESCE( (SELECT SUM(amount) FROM ledger WHERE account_id = accounts.id), 0) AS SIGNED)
                ) AS amount_value,
                id
            FROM 
                accounts
            WHERE
                id = ?
            LIMIT 1;
    ",
    )
    .bind(account_id.clone())
    // .bind(account_id)
    .fetch_optional(&conn)
    .await?;

    Result::Ok(item)
}

/// This method returns the balance of the
pub async fn by_user_id(
    user_id: String,
    db: DbHandle,
) -> Result<Vec<crate::models::AccountBalance>, sqlx::Error> {
    let conn = db.connect().await.unwrap();

    let items = sqlx::query_as::<_, crate::models::AccountBalance>(
        "
            SELECT
                id,
                currency AS amount_currency,
                (
                    CAST(COALESCE( (SELECT SUM (amount) FROM ledger WHERE user_id = ? ), 0) AS SIGNED)
                ) AS amount_value
            FROM accounts WHERE user_id = ?;
        ",
    )
    .bind(user_id.clone())
    .bind(user_id)
    .fetch_all(&conn)
    .await?;

    Result::Ok(items)
}
