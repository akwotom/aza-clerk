/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic for retrieving the transaction history of an account
*/

use crate::{logic::db::DbHandle, models::transaction::Transaction};

/// This method returns the list of transactions for the given account, with pagination.
/// The `per_page` param tells us how many items in the transaction history list ought to be returned at once.
/// The `page` param tells us which page the caller wants to read, assuming he has previously read the other pages.
pub async fn by_account_id(
    account_id: String,
    per_page: i32,
    page: i32,
    db: &DbHandle,
) -> Result<Vec<Transaction>, sqlx::Error> {
    let items: Vec<Transaction> = sqlx::query_as::<_, Transaction>(
        "
            SELECT * FROM transactions 
            WHERE 
                transfer_src_account_id = ? OR
                transfer_bene_account_id = ? OR
                top_up_bene_account_id = ?
            LIMIT ?
            OFFSET ?;
    ",
    )
    .bind(account_id.clone())
    .bind(account_id.clone())
    .bind(account_id)
    .bind(per_page)
    .bind((page - 1) * per_page)
    .fetch_all(&db.connect().await?)
    .await?;

    Result::Ok(items)
}

/// This method returns the transaction history for all accounts owned by the given user.
pub async fn by_user_id(
    user_id: String,
    per_page: i32,
    page: i32,
    db: &DbHandle,
) -> Result<Vec<Transaction>, sqlx::Error> {
    let items: Vec<Transaction> = sqlx::query_as::<_, Transaction>(
        "
            SELECT * FROM transactions 
            WHERE 
                transfer_src_user_id = ? OR
                transfer_bene_user_id = ? OR
                top_up_bene_user_id = ?
            LIMIT ?
            OFFSET ?;
    ",
    )
    .bind(user_id.clone())
    .bind(user_id.clone())
    .bind(user_id)
    .bind(per_page)
    .bind((page - 1) * per_page)
    .fetch_all(&db.connect().await?)
    .await?;

    Result::Ok(items)
}
