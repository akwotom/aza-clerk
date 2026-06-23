/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic that validates that a piece of string is of the required UUID-v4 format.
*/

use crate::http::response::AzaResponse;

pub fn check_uuid_string(string: &String) -> Result<(), AzaResponse<()>> {
    let regexp = regex::Regex::new(
        "(?i)^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$",
    )
    .unwrap();
    if !regexp.is_match(string) {
        return Result::Err(AzaResponse::<()>::Failed {
            code: "invalid_input".to_string(),
            http_code: axum::http::StatusCode::BAD_REQUEST,
            message: format!("The following string doesn't follow the UUID-v4 format. {string}"),
        });
    }
    Result::Ok(())
}
