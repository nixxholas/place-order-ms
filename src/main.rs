use tracing::info;

mod actions;

pub struct Config {
    pub order_ms_url: String,
    pub shipping_ms_url: String,
    pub product_ms_url: String
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    // Get environment variables
    dotenv::dotenv().ok();

    let config = Config {
        order_ms_url: std::env::var("ORDER_MS").expect("ORDER_MS must be set"),
        shipping_ms_url: std::env::var("SHIPPING_MS").expect("SHIPPING_MS must be set"),
        product_ms_url: std::env::var("PRODUCT_MS").expect("PRODUCT_MS must be set")
    };
    let env_type = std::env::var("ENVIRONMENT").expect("ENVIRONMENT must be set");

    if env_type != "Production" && env_type != "Staging" {
        let machine_user = whoami::username().to_uppercase();
        info!("Welcome to development {}!", machine_user);
    }

    // Spin up the API server
    actions::serve(config).await;
}