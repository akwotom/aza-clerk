/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module is the entry point of every other module concerned only with logic (behaviour).
    This is loosely the equivalent of a controller.
*/

pub mod db;
pub mod get_transaction_history;
mod sql_procedures;
pub mod transfer;

use db::DbHandle;

pub mod top_up;

pub mod sql_utils;

pub mod get_account_balance;

mod uuid_string;

pub mod foreign_exchange;

pub use uuid_string::*;

pub async fn init(db: &DbHandle) -> Result<(), sqlx::Error> {
    sql_procedures::init_sql_procedures(db).await
}
