/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic for performing account balance top-up.
*/

use crate::{
    logic::{check_uuid_string, db::DbHandle},
    models::Amount,
};

/// This method begins the process of topping the user's account balance
/// An external id needs to be passed, in order to prevent duplicates
pub async fn by_user_id(
    user_id: String,
    transaction_id: String,
    amount: Amount,
    db: &DbHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = db.connect().await?;
    
    check_uuid_string(&transaction_id)?;

    sqlx::query("CALL credit_user (?, ?, ?, ?)")
        .bind(transaction_id)
        .bind(user_id)
        .bind(amount.value)
        .bind(amount.currency)
        .execute(&connection)
        .await?;

    Result::Ok(())
}
