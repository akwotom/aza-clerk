/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module defines the data structure of an entry in the ledger system.
    It defines a single immutable finalized bookkeeping operation.
    Our system is a double-entry ledger system. Therefore, our entries are simple
*/

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct LedgerEntry {
    /// The account that this ledger entry belongs to.
    pub account_id: String,
    /// This is the id of the user that owns the account that this entry involves.
    /// Yes, we had to duplicate the field, in order to save on needless join queries.
    pub user_id: String,
    pub amount: i32,
    /// This is auto-incremented by the database.
    sequence_id: i32,
    /// The transaction that this ledger entry targets
    transaction_id: String,

    /// The UTC time when this entry was created.
    created_at_utc: String,
}

impl LedgerEntry {
    pub(crate) fn sql_create_statement() -> String {
        "
            CREATE TABLE IF NOT EXISTS ledger (
                sequence_id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                account_id TEXT NOT NULL,
                amount INT NOT NULL,
                transaction_id TEXT,
                user_id TEXT NOT NULL,
                created_at_utc DATETIME DEFAULT (UTC_TIMESTAMP())
            );
        "
        .to_string()
    }
}
