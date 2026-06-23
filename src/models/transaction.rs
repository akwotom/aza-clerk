/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module defines the data structure of a transaction on the system.
    A transaction represents the high-level intention of a user, e.g transferring money, or topping up his account.

*/

use crate::models::Amount;

#[derive(
    serde::Serialize, serde::Deserialize, Clone, sqlx::FromRow, sqlx::Decode, std::fmt::Debug,
)]
pub struct Transaction {
    /// A unique long-string transaction id
    pub id: String,
    pub action: Action,
    #[sqlx(flatten)]
    pub amount: Amount,
    /// If present (for a transfer), the it represents the user who's sending the money.
    pub transfer_src_user_id: Option<String>,
    pub transfer_src_account_id: Option<String>,

    pub transfer_bene_user_id: Option<String>,
    pub transfer_bene_account_id: Option<String>,

    /// If present, it represents the beneficiary of a transaction meant to top_up the user's account balance.
    pub top_up_bene_user_id: Option<String>,
    pub top_up_bene_account_id: Option<String>,

    created_at_utc: chrono::DateTime<chrono::Utc>,

    pub status: Status,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    TopUp,
    Transfer,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Successful,
    Failed,
    Reversed,
}

impl Transaction {
    pub(crate) fn sql_create_statement() -> String {
        "
            CREATE TABLE IF NOT EXISTS transactions (
                id VARCHAR(64) NOT NULL PRIMARY KEY,
                action TEXT NOT NULL,
                status ENUM ('pending', 'successful', 'failed', 'reversed') NOT NULL,
                amount_value INT NOT NULL,
                amount_currency TEXT NOT NULL,

                transfer_src_user_id VARCHAR(64),
                transfer_src_account_id VARCHAR(64),

                transfer_bene_user_id VARCHAR(64),
                transfer_bene_account_id VARCHAR(64),

                top_up_bene_user_id VARCHAR(64),
                top_up_bene_account_id VARCHAR(64),

                created_at_utc DATETIME DEFAULT (UTC_TIMESTAMP())
            );
        "
        .to_string()
    }
}

impl sqlx::Type<sqlx::MySql> for Status {
    fn type_info() -> <sqlx::MySql as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::MySql>>::type_info()
    }
    fn compatible(ty: &<sqlx::MySql as sqlx::Database>::TypeInfo) -> bool {
        <String as sqlx::Type<sqlx::MySql>>::compatible(ty)
    }
}

impl sqlx::Type<sqlx::MySql> for Action {
    fn type_info() -> <sqlx::MySql as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::MySql>>::type_info()
    }
    fn compatible(ty: &<sqlx::MySql as sqlx::Database>::TypeInfo) -> bool {
        <String as sqlx::Type<sqlx::MySql>>::compatible(ty)
    }
}
impl<'r> sqlx::Decode<'r, sqlx::MySql> for Action {
    fn decode(value: sqlx::mysql::MySqlValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let raw_string = <String as sqlx::Decode<'r, sqlx::MySql>>::decode(value)?;

        Result::Ok(match raw_string.as_str() {
            "top_up" => Action::TopUp,
            "transfer" => Action::Transfer,
            _ => {
                return Result::Err(sqlx::error::BoxDynError::from("Invalid value for Action"));
            }
        })
    }
}

impl<'r> sqlx::Decode<'r, sqlx::MySql> for Status {
    fn decode(value: sqlx::mysql::MySqlValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let raw_string = <String as sqlx::Decode<'r, sqlx::MySql>>::decode(value)?;

        Result::Ok(match raw_string.as_str() {
            "failed" => Status::Failed,
            "pending" => Status::Pending,
            "successful" => Status::Successful,
            "reversed" => Status::Reversed,
            _ => {
                return Result::Err(sqlx::error::BoxDynError::from(format!(
                    "Invalid value for Status '{raw_string}'"
                )));
            }
        })
    }
}
