/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module defines the data structure of an account.
    By Account, we're talking of a financial account.
*/

use crate::models::Amount;

#[derive(Clone, Debug, sqlx::FromRow, serde::Serialize)]
pub(crate) struct Account {
    /// Unique generated id for the account.
    pub id: String,
    /// The user in possession of this account.
    pub user_id: String,
    /// The currency in which all transactions of the account are defined.
    pub currency: String,
}

#[derive(Clone, Debug, sqlx::FromRow, serde::Serialize)]
pub(crate) struct AccountBalance {
    pub id: String,
    #[sqlx(flatten)]
    amount: Amount,
}

impl Account {
    pub(crate) fn sql_create_statement() -> String {
        "
            CREATE TABLE IF NOT EXISTS accounts (
                id VARCHAR(64) NOT NULL PRIMARY KEY DEFAULT (UUID()),
                user_id TEXT NOT NULL,
                currency TEXT NOT NULL,
                created_at_utc DATETIME DEFAULT (UTC_TIMESTAMP())
            );
        "
        .to_string()
    }
}
