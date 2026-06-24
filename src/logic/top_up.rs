/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic for performing account balance top-up.
*/

use crate::{
    http::response::AzaResponse,
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

    match sqlx::query("CALL credit_user (?, ?, ?, ?)")
        .bind(transaction_id)
        .bind(user_id)
        .bind(amount.value)
        .bind(amount.currency)
        .execute(&connection)
        .await
    {
        Result::Ok(_) => {}
        Result::Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if let Option::Some(err_code) = db_err.code() {
                    match err_code.to_string().as_str() {
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
