/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module defines a blueprint for standard HTTP responses, be it successful, or failed.
*/

use serde::ser::SerializeStruct;

#[derive(std::fmt::Debug, Clone)]
pub enum AzaResponse<T>
where
    T: serde::Serialize,
    T: std::fmt::Debug,
{
    Success {
        data: T,
    },
    Failed {
        code: String,
        message: String,
        http_code: axum::http::StatusCode,
    },
}

impl<T> axum::response::IntoResponse for AzaResponse<T>
where
    T: std::fmt::Debug,
    T: serde::Serialize,
{
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        match self {
            AzaResponse::Failed {
                code,
                message,
                http_code,
            } => {
                let mut headers = axum::http::HeaderMap::new();
                headers.append(
                    "Content-Type",
                    axum::http::HeaderValue::from_str("application/json").unwrap(),
                );

                (
                    http_code,
                    headers,
                    serde_json::json!({
                        "success": false,
                        "error": {
                            "code": code,
                            "message": message,
                        }
                    })
                    .to_string(),
                )
                    .into_response()
            }
            AzaResponse::Success { data: body } => {
                let mut headers = axum::http::HeaderMap::new();
                headers.append(
                    "Content-Type",
                    axum::http::HeaderValue::from_str("application/json").unwrap(),
                );
                (
                    axum::http::StatusCode::OK,
                    headers,
                    serde_json::json!({
                        "success": true,
                        "data": &body,
                    })
                    .to_string(),
                )
                    .into_response()
            }
        }
        .into_response()
    }
    //
}

#[derive(serde::Serialize, std::fmt::Debug)]
pub struct PaginatedData<T> {
    pub per_page: i32,
    pub page: i32,
    pub items: Vec<T>,
}

impl<T> std::fmt::Display for AzaResponse<T>
where
    T: std::fmt::Debug,
    T: serde::Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Failed {
                    code,
                    http_code,
                    message,
                } => {
                    format!("Error: {message}\nCode: {code}\nHTTP {http_code}\n")
                }
                Self::Success { data } => {
                    format!("{data:?}\nStatus: 200\n")
                }
            }
        )
    }
}

impl<T> std::error::Error for AzaResponse<T>
where
    T: std::fmt::Debug,
    T: serde::Serialize,
{
}

impl<T> serde::Serialize for AzaResponse<T>
where
    T: std::fmt::Debug,
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        //
        let mut state = serializer.serialize_struct("AzaResponse", 2)?;

        match self {
            AzaResponse::Success { data: body } => state.serialize_field("data", body)?,
            AzaResponse::Failed { code, message, .. } => state.serialize_field(
                "error",
                &std::collections::HashMap::from([("message", message), ("code", code)]),
            )?,
        };

        state.end()
    }
}
