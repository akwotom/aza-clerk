/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This contains the logic for serving requests that have to do with transfers
*/

use axum::response::IntoResponse;

use crate::{
    http::{response::AzaResponse, server::RouterState},
    logic,
    models::Amount,
};

/// This method returns a router configured to follow the logic of the transfer module.
pub(crate) fn transfer_router() -> axum::Router<RouterState> {
    axum::Router::new().route(
        "/",
        axum::routing::post(
            async |axum::extract::State(state): axum::extract::State<RouterState>,
                   axum::extract::Json(demand): axum::Json<StartTransferDemand>| {
                match logic::transfer::transfer_money(
                    demand.id,
                    demand.amount,
                    demand.funding_user_id,
                    demand.recipient_user_id,
                    &state.db,
                )
                .await
                {
                    Result::Ok(_) => {}
                    Result::Err(e) => {
                        if let Option::Some(err) = e.as_ref().downcast_ref::<AzaResponse<()>>() {
                            return err.clone().into_response();
                        }
                        panic!("{e}");
                    }
                };

                AzaResponse::Success { data: () }.into_response()
            },
        ),
    )
    .route("/{id}/complete", axum::routing::post(async |axum::extract::State(state): axum::extract::State<RouterState>, axum::extract::Path(id): axum::extract::Path<String>|{
        match logic::transfer::complete_transfer(id, &state.db).await {
            Result::Ok(_)=>{},
            Result::Err(e)=>{
                if let Option::Some(err)= e.downcast_ref::<AzaResponse::<()>>(){
                    return err.clone().into_response();
                }
                panic!("{e}");
            }
        };
        AzaResponse::Success {
            data: ()
        }.into_response()
    }))
}

#[derive(Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct StartTransferDemand {
    id: String,
    amount: Amount,
    funding_user_id: String,
    recipient_user_id: String,
}
