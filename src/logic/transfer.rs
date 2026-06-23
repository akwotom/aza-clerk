/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic that governs the transfer of funds between accounts.
*/

use crate::{
    logic::{check_uuid_string, db::DbHandle},
    models::Amount,
};

/// This method begins the process of transferring money from one user to another.
pub async fn transfer_money(
    txn_id: String,
    amount: Amount,
    funding_user_id: String,
    recipient_user_id: String,
    db: &DbHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    // First, we need to be sure that the user has sufficient balance.
    // There's a stored procedure for just beginning transfers,
    // There's another stored procedure for completing the transfer
    // Finally, there's a stored procedure for rejecting a transfer
    // NOTE: actions that require db locking happen

    check_uuid_string(&txn_id)?;

    let outcome = sqlx::query("CALL begin_transfer (?, ?, ?, ?, ?);")
        .bind(txn_id)
        .bind(funding_user_id)
        .bind(recipient_user_id)
        .bind(amount.value)
        .bind(amount.currency)
        .execute(&(db.connect().await?))
        .await?;

    // TODO: Handle the exceptional outcomes of the begin_transfer procedure.
    println!("Results of transfer_money operation.\n{outcome:?}\n");

    Result::Ok(())
}
