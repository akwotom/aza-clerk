/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic that handles HTTP traffic intended for the account management feature
*/

use axum::response::IntoResponse;

use crate::{
    http::{response::{AzaResponse, PaginatedData}, server::RouterState},
    logic::{self},
    models::Amount,
};

/// This method returns a router configured to serve users based on the logic of the account module.
pub fn account_router() -> axum::Router<RouterState> {
    let router = axum::Router::new();

    router.route(
        "/{id}/balance",
        axum::routing::get(
            async |axum::extract::Path(account_id): axum::extract::Path<String>,
                axum::extract::State(state): axum::extract::State<RouterState>| {
                // This end-point is called when a service wants to get the balance of a account, by id.

                let value = match logic::get_account_balance::by_account_id(account_id.clone(), state.db)
                    .await {
                        Result::Ok(v)=> v,
                        Result::Err(e)=> panic!("{e}"), // Don't fear, there's a panic-catching middleware
                    };

                match value {
                    Option::Some(account) => AzaResponse::Success { data: account },
                    Option::None => AzaResponse::Failed {
                        code: "account.entity_not_found".to_string(),
                        message: format!("The account with id {account_id} was not found"),
                        http_code: axum::http::StatusCode::NOT_FOUND,
                    },
                }
                .into_response()
            },
        ),
    )
    
    .route("/{id}/history", axum::routing::get(async |axum::extract::State(state): axum::extract::State<RouterState>,axum::extract::Query(query): axum::extract::Query<TransactionHistoryQuery>, axum::extract::Path(id):  axum::extract::Path<String>, | {
        
        let TransactionHistoryQuery {
            page,
            per_page,
        } = query;

        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(50);
        
        let results = match logic::get_transaction_history::by_account_id(
            id, 
            per_page, 
            page, 
            &state.db
        ).await {
            Result::Ok(v)=> v,
            Result::Err(e)=> {
                // TODO: Proper error handling for all possible database situations.
                    panic!("{e}")
            }
        };


        AzaResponse::Success {
            data: PaginatedData {
                items: results,
                per_page,
                page,
            }
        }.into_response()
        
    }))
    .route(
        "/user/{id}/history",
        axum::routing::get(async |axum::extract::State(state): axum::extract::State<RouterState>,axum::extract::Query(query): axum::extract::Query<TransactionHistoryQuery>, axum::extract::Path(user_id):  axum::extract::Path<String>, |  {
    

        let TransactionHistoryQuery {
            page,
            per_page,
        } = query;

        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(50);
        
        let results = logic::get_transaction_history::by_user_id(
            user_id, 
            per_page, 
            page, 
            &state.db
        ).await.unwrap();


        AzaResponse::Success {
            data: PaginatedData {
                items: results,
                per_page,
                page,
            }
        }
    }))
    .route("/user/{id}/balances", axum::routing::get(async |axum::extract::State(state): axum::extract::State<RouterState>,  axum::extract::Path(user_id):  axum::extract::Path<String>, |{
        // This end-point is called when a service wants to get the balance of a account, by id.

        let value = logic::get_account_balance::by_user_id(user_id, state.db)
            .await
            .unwrap();

        AzaResponse::Success {
            data: value,
        }
    }))
    .route(
        "/user/{id}/top-up",
        axum::routing::post(
            async |axum::extract::State(state): axum::extract::State<RouterState>, axum::extract::Path(user_id):axum::extract::Path<String>, axum::Json(payload): axum::Json<AccountTopUpDemand>| {
                
                match logic::top_up::by_user_id(user_id, payload.transaction_id, payload.amount,  &state.db).await {
                    Result::Ok(v)=> v,
                    Result::Err(e)=> {
                        let copy = e.as_ref();
                        
                        if let Option::Some(err) = e.downcast_ref::<AzaResponse<()>>() {
                            println!("Now responding with intentional error.");
                            return err.clone().into_response();
                        }else {
                            panic!("{copy}")
                        }
                    }
                };
                
                AzaResponse::Success {
                    data: ()
                }.into_response()
            }
        )
    )

    
}

#[derive(Clone, serde::Deserialize)]
#[allow(dead_code)]
struct AccountTopUpDemand {
    transaction_id: String,
    amount: Amount,
}


#[derive(Clone, serde::Deserialize)]
struct TransactionHistoryQuery {
    per_page: Option<i32>,
    page: Option<i32>,
}