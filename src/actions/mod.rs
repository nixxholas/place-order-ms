use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use axum::{Router, error_handling::HandleErrorLayer, http::StatusCode};
use axum::extract::Extension;
use tower::{ServiceBuilder, BoxError};
use tower_http::trace::TraceLayer;
use serde::{Deserialize};
use crate::Config;

/// Import all the api modules
pub mod order;


/// Core type which handler functions can access common API state
/// This can be accessed by adding `Extension<ApiContext>` to a handler function's parameter
struct ApiContext {
    config: Arc<Config>,
}

// Kickstart the server and start listening for requests
pub async fn serve(config: Config) -> () {
    // Create a shared state context
    let api_context = Arc::new(ApiContext {
        config: Arc::new(config)
    });

    // Create a new router
    let app = api_router().layer(
        ServiceBuilder::new()
            .layer(Extension(api_context))
            .layer(HandleErrorLayer::new(|error: BoxError| async move {
                if error.is::<tower::timeout::error::Elapsed>() {
                    Ok(StatusCode::REQUEST_TIMEOUT)
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    ))
                }
            }))
            .timeout(Duration::from_secs(10))
            .layer(TraceLayer::new_for_http())
            .into_inner(),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

fn api_router() -> Router {
    // This is the order that the modules were authored in.
    order::router()
}