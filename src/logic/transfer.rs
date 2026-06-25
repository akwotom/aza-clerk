/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic that governs the transfer of funds between accounts.
*/

use crate::{
    http::{response::AzaResponse, server::RouterState},
    logic::{check_uuid_string, db::DbHandle},
    models::Amount,
};

/// This method begins the process of transferring money from one user to another.
pub async fn transfer_money(
    txn_id: String,
    amount: Amount,
    funding_user_id: String,
    recipient_user_id: String,
    funding_currency: Option<String>,
    state: &RouterState,
    // fx: ForeignExchange,
) -> Result<(), Box<dyn std::error::Error>> {
    // First, we need to be sure that the user has sufficient balance.
    // There's a stored procedure for just beginning transfers,
    // There's another stored procedure for completing the transfer
    // Finally, there's a stored procedure for rejecting a transfer
    // NOTE: actions that require db locking happen

    check_uuid_string(&txn_id)?;

    let mut funding_amount = amount.clone();

    // Now, if the transaction is to happen in a different currency, let's compute the transaction fees
    if let Option::Some(funding_currency) = funding_currency {
        funding_amount = Amount {
            currency: funding_currency.clone(),
            value: match state.fx.convert(funding_amount, funding_currency).await {
                Result::Ok(v) => v,
                Result::Err(e) => return Result::Err(e),
            },
        }
    }

    match sqlx::query("CALL begin_transfer (?, ?, ?, ?, ?, ?, ?);")
        .bind(txn_id)
        .bind(funding_user_id)
        .bind(recipient_user_id)
        .bind(amount.value)
        .bind(amount.currency)
        .bind(funding_amount.value)
        .bind(funding_amount.currency)
        .execute(&(state.db.connect().await?))
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
                                http_code: axum::http::StatusCode::NOT_ACCEPTABLE,
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
pub async fn complete_transfer(
    txn_id: String,
    db: &DbHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    match sqlx::query("CALL complete_transfer (?)")
        .bind(txn_id)
        .execute(&db.connect().await?)
        .await
    {
        Result::Ok(_) => {}
        Result::Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if let Option::Some(err_code) = db_err.code() {
                    match err_code.to_string().as_str() {
                        "45010" => {
                            return Result::Err(Box::from(AzaResponse::<()>::Failed {
                                code: "bad_state".to_string(),
                                message: "The transaction is not in a state that can be completed."
                                    .to_string(),
                                http_code: axum::http::StatusCode::NOT_ACCEPTABLE,
                            }));
                        }
                        "45011" => {
                            return Result::Err(Box::from(AzaResponse::<()>::Failed {
                                code: "entity_not_found".to_string(),
                                message: "The transaction that is to be completed is either not found, or not a transfer. "
                                    .to_string(),
                                http_code: axum::http::StatusCode::NOT_FOUND,
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
