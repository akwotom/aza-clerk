/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic that governs the transfer of funds between accounts.
*/

use crate::{
    http::response::AzaResponse,
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

    match sqlx::query("CALL begin_transfer (?, ?, ?, ?, ?);")
        .bind(txn_id)
        .bind(funding_user_id)
        .bind(recipient_user_id)
        .bind(amount.value)
        .bind(amount.currency)
        .execute(&(db.connect().await?))
        .await
    {
        Result::Ok(_) => {}
        Result::Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if let Option::Some(err_code) = db_err.code() {
                    match err_code.to_string().as_str() {
                        "45008" => {
                            return Result::Err(Box::from(AzaResponse::<()>::Failed {
                                code: "balance_insufficient".to_string(),
                                message: "The user's balance is insufficient".to_string(),
                                http_code: axum::http::StatusCode::BAD_REQUEST,
                            }));
                        }
                        "45009" => {
                            // Transaction already exists
                            return Result::Err(Box::from(AzaResponse::<()>::Failed {
                                code: "duplicate_ref".to_string(),
                                message: "A transaction already exists with the given id"
                                    .to_string(),
                                http_code: axum::http::StatusCode::CONFLICT,
                            }));
                        }
                        _ => {}
                    };

                    return Result::Err(Box::from(e));
                }
            }
            return Result::Err(Box::from(e));
        }
    };

    Result::Ok(())
}

/// This method completes a pending transfer that was already begun.
pub async fn complete_transfer(txn_id: String) {}
