/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic for managing the HTTP server
*/

use crate::{
    http::{account::account_router, response::AzaResponse, transfer::transfer_router},
    logic::{db::DbHandle, foreign_exchange::ForeignExchange},
};

use axum::response::IntoResponse;

pub async fn create_server(http_port: i32, db: DbHandle, fx: ForeignExchange) -> () {
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{http_port}"))
        .await
        .unwrap();

    let state = RouterState { db, fx: fx };

    fn cors_headers() -> axum::http::HeaderMap {
        let mut headers = axum::http::HeaderMap::new();

        headers.append(
            "Access-Control-Allow-Origin",
            axum::http::HeaderValue::from_str("*").unwrap(),
        );
        headers.append(
            "Access-Control-Allow-Methods",
            axum::http::HeaderValue::from_str("*").unwrap(),
        );
        headers.append(
            "Access-Control-Allow-Headers",
            axum::http::HeaderValue::from_str("*").unwrap(),
        );

        headers
    }

    let root_router = axum::Router::<RouterState>::new()
        .nest(
            "/sapi",
            axum::routing::Router::new()
                .nest("/account", account_router())
                .nest("/transfer", transfer_router()),
        )
        .layer(axum::middleware::from_fn(
            |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| async {
                if req.method().to_string().to_uppercase() == "OPTIONS" {
                    return (axum::http::StatusCode::OK, cors_headers(), "It works!")
                        .into_response();
                }
                next.run(req).await
            },
        ))
        .layer(axum::middleware::from_fn(
            |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| async {
                let mut resp = next.run(req).await;
                resp.headers_mut().extend(cors_headers());

                resp
            },
        ))
        .layer(tower_http::catch_panic::CatchPanicLayer::custom(
            |e: Box<dyn std::any::Any + Send>| {
                if let Result::Ok(value) = e.downcast::<AzaResponse<()>>() {
                    return value.into_response();
                }
                AzaResponse::<()>::Failed {
                    code: "server_error".to_string(),
                    http_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    message: "Server Error!".to_string(),
                }
                .into_response()
            },
        ))
        .with_state(state);

    axum::serve(listener, root_router).await.unwrap();
}

#[derive(Clone)]
pub struct RouterState {
    pub(crate) db: DbHandle,
    pub(crate) fx: ForeignExchange,
}
