/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic for initializing vital components related to database models.
*/

use crate::{
    logic::{db::DbHandle, sql_utils},
    models::{Account, ledger::LedgerEntry, transaction::Transaction},
};

/// This method initializes all database structures
pub async fn init(db: &DbHandle) -> Result<(), sqlx::Error> {
    sql_utils::run_multiple_sql(
        &[
            LedgerEntry::sql_create_statement().as_str(),
            Transaction::sql_create_statement().as_str(),
            Account::sql_create_statement().as_str(),
        ],
        db,
    )
    .await
}
