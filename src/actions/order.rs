use std::sync::Arc;
use serde::{Deserialize, Serialize};
use axum::http::{StatusCode};
use axum::{Router, Json};
use axum::body::HttpBody;
use axum::extract::{Extension, Path, Query};
use axum::routing::{get};
use serde_json::Value;
use tracing::{debug, error, info};
use tracing::field::debug;
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
    data: CreatedOrder
}

#[derive(Clone, Serialize, Deserialize)]
struct CreatedShipmentResult {
    code: u32,
    data: CreatedShipment,
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
    // date_created: chrono::NaiveDateTime,
    dc_order_qty: i32
}

#[derive(Clone, Serialize, Deserialize)]
struct Preshipment {
    email: String,
    handled_by: String
}

#[derive(Clone, Serialize, Deserialize)]
struct CreatedShipment {
    order_id: u64,
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
                            let submitted_order_res_body = submitted_order_res.text().await.unwrap();
                            let submitted_order: CreatedOrderResult =
                                serde_json::from_str(&submitted_order_res_body).unwrap();
                            let submitted_shipping_request = reqwest::Client::new()
                                .post(format!("{}/shipping/create/{}", shipping_ms_url,
                                              submitted_order.data.order_id))
                                .json(&Preshipment {
                                    email: order.email.to_string(),
                                    handled_by: order.handled_by.to_string()
                                })
                                .send().await;

                            if let Ok(shipment_response_result) = submitted_shipping_request {
                                if shipment_response_result.status() == StatusCode::OK ||
                                    shipment_response_result.status() == StatusCode::CREATED {
                                    let _submitted_shipment_response: CreatedShipmentResult =
                                        serde_json::from_str(&shipment_response_result.text().await.unwrap()).unwrap();
                                    let new_product_change = ProductUpdate {
                                        product_name: order.product_name,
                                        incoming_stocks: order.incoming_stocks,
                                        stocks: order.stocks
                                    };

                                    let product_call = reqwest::Client::new()
                                        .put(format!("{}/product/update/one", product_ms_url))
                                        .json(&new_product_change)
                                        .send().await;

                                    if let Ok(product_update_response) = product_call {
                                        if product_update_response.status() == StatusCode::OK ||
                                            product_update_response.status() == StatusCode::CREATED {
                                            return Ok(StatusCode::OK)
                                        }
                                    }
                                } else {
                                    return Err(shipment_response_result.status())
                                }
                            } else {
                                return Err(submitted_shipping_request.unwrap_err().status().unwrap())
                            }
                        } else {
                            return Err(submitted_order_res.status())
                        }
                    }

                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                })
            })
            .collect();

        for order_task in order_tasks {
            let _result = order_task.await;
        }
    } else {
        return Err(StatusCode::NO_CONTENT)
    }

    Ok(StatusCode::OK)
}