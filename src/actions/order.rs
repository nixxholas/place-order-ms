use std::sync::Arc;
use serde::{Deserialize, Serialize};
use axum::http::{StatusCode};
use axum::{Router, Json};
use axum::extract::{Extension, Path, Query};
use axum::routing::{get};
use tracing::{error, info};
use crate::actions::{ApiContext};

pub fn router() -> Router {
    Router::new()
        .route("/place_order", get(get_place_order).post(place_order))
}

#[derive(Clone, Serialize, Deserialize)]
struct Order {
    dc: String,
    dc_order_qty: i32,
    email: String,
    handled_by: String,
    incoming_stocks: i32,
    product_id: String,
    product_name: String,
    quantity: i32,
    retailer: String,
    stocks: i32,
    supplier: String
}

#[derive(Clone, Serialize, Deserialize)]
struct CreatedOrderResult {
    code: u32,
    data: String
}

#[derive(Clone, Serialize, Deserialize)]
struct CreatedShipmentResult {
    code: u32,
    data: String,
    message: String
}

#[derive(Clone, Serialize, Deserialize)]
struct CreatedOrder {
    order_id: u32,
    product_name: String,
    quantity: i32,
    retailer: String,
    supplier: String,
    order_status: String,
    dc: String,
    date_created: chrono::NaiveDateTime,
    dc_order_qty: i32
}

#[derive(Clone, Serialize, Deserialize)]
struct CreatedShipment {
    order_id: u64,
    delivery_date: chrono::NaiveDateTime,
    shipping_status: String,
    handled_by: String
}

#[derive(Clone, Serialize, Deserialize)]
struct ProductUpdate {
    product_name: String,
    incoming_stocks: i32,
    stocks: i32
}

async fn get_place_order() -> Result<StatusCode, StatusCode> {
    Err(StatusCode::METHOD_NOT_ALLOWED)
}

/// Get all close account
async fn place_order(ctx: Extension<Arc<ApiContext>>, Json(payload): Json<Vec<Order>>) -> Result<StatusCode, StatusCode> {
    if !payload.is_empty() {
        let orders = payload.to_vec();
        info!("Processing {:?} orders", orders.len());

        let order_tasks: Vec<_> = orders
            .into_iter()
            .map(|order| {
                let order_ms_url = (&ctx.config.order_ms_url).clone();
                let shipping_ms_url = (&ctx.config.shipping_ms_url).clone();
                let product_ms_url = (&ctx.config.product_ms_url).clone();
                tokio::spawn(async move {
                    let order_response = reqwest::Client::new()
                        .post(format!("{}/order/create", order_ms_url))
                        .json(&order)
                        .send().await;

                    if let Ok(submitted_order_res) = order_response {
                        if submitted_order_res.status() == StatusCode::OK ||
                            submitted_order_res.status() == StatusCode::CREATED {
                            let submitted_order_response: Result<CreatedOrder, _> =
                                serde_json::from_str(&submitted_order_res.json::<CreatedOrderResult>().await.unwrap().data);

                            if let Ok(submitted_order) = submitted_order_response {
                                let submitted_shipping_request = reqwest::Client::new()
                                    .post(format!("{}/shipping/create/{}", shipping_ms_url,
                                                  submitted_order.order_id))
                                    .send().await;

                                if let Ok(shipment_response_result) = submitted_shipping_request {
                                    if shipment_response_result.status() == StatusCode::OK ||
                                        shipment_response_result.status() == StatusCode::CREATED {
                                        let submitted_shipment_response: Result<CreatedShipment, _> =
                                            serde_json::from_str(&shipment_response_result.json::<CreatedShipmentResult>().await.unwrap().data);

                                        if let Ok(_) = submitted_shipment_response {
                                            let new_product_change = ProductUpdate {
                                                product_name: submitted_order.product_name,
                                                incoming_stocks: order.incoming_stocks,
                                                stocks: order.stocks
                                            };

                                            let product_call = reqwest::Client::new()
                                                .post(format!("{}/product/update/one", product_ms_url))
                                                .json(&new_product_change)
                                                .send().await;

                                            if let Ok(product_update_resposne) = product_call {
                                                if product_update_resposne.status() == StatusCode::OK ||
                                                    product_update_resposne.status() == StatusCode::CREATED {
                                                    return Ok(StatusCode::OK)
                                                }
                                            }
                                         }
                                    }
                                }
                            }
                        }
                    }

                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                })
            })
            .collect();
    }

    Ok(StatusCode::OK)
    // match rs {
    //     Ok(mut rs) => {
    //         // let mut data: Vec<FleetCloseAccount> = Vec::new();
    //         // while rs.next_row() {
    //         //     data.push(FleetCloseAccount {
    //         //         user: rs.get_string(0).unwrap().unwrap(),
    //         //         ship_mint: rs.get_string(1).unwrap().unwrap(),
    //         //         timestamp: rs.get_i64(2).unwrap().unwrap(),
    //         //     });
    //         // }
    //     }
    //     Err(e) => {
    //         error!("{:?}", e);
    //         Err(StatusCode::INTERNAL_SERVER_ERROR)
    //     }
    // }
}