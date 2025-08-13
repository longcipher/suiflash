mod collectors;
mod config;
mod executors;
mod strategies;

#[cfg(test)]
mod tests;

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use collectors::ProtocolDataCollector;
use config::{Config, FlashLoanRequest, FlashLoanResponse, ProtocolsResponse, StatusResponse};
use executors::FlashLoanExecutor;
use eyre::Result;
use strategies::FlashLoanStrategy;
use tokio::net::TcpListener;
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    config: Config,
    strategy: FlashLoanStrategy,
    executor: FlashLoanExecutor,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env()?;
    info!("Starting SuiFlash bot with config: {:?}", config);
    // Touch individual fields to avoid dead_code warnings until they are fully wired.
    let _touch = (
        &config.sui_rpc_url,
        &config.private_key,
        &config.sui_flash_package_id,
        &config.sui_flash_config_object_id,
        &config.navi_package_id,
        &config.bucket_package_id,
        &config.scallop_package_id,
    );

    // Initialize components
    let collector = ProtocolDataCollector::new(config.clone());
    let strategy = FlashLoanStrategy::new(config.clone(), collector.clone());
    let executor = FlashLoanExecutor::new(config.clone());

    // Start background data collection
    let collector_handle = {
        let collector = collector.clone();
        tokio::spawn(async move {
            collector.start_background_collection().await;
        })
    };

    // Create app state
    let app_state = AppState {
        config: config.clone(),
        strategy,
        executor,
    };

    // Build the router
    let app = Router::new()
        .route("/flashloan", post(handle_flash_loan))
        .route("/protocols", get(handle_protocols))
        .route("/status", get(handle_status))
        .route("/health", get(handle_health))
        .with_state(app_state);

    // Start the server
    let addr = format!("0.0.0.0:{}", config.server_port);
    info!("Starting server on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    // Clean up background tasks
    collector_handle.abort();

    Ok(())
}

async fn handle_flash_loan(
    State(state): State<AppState>,
    Json(request): Json<FlashLoanRequest>,
) -> Result<Json<FlashLoanResponse>, StatusCode> {
    info!("Received flash loan request: {:?}", request);
    info!("Current strategy mode: {}", state.config.strategy);

    // Determine protocol if explicit routing requested
    let execution_plan = if let Some(p) = &request.explicit_protocol {
        match state.strategy.override_protocol(&request, *p).await {
            Ok(plan) => plan,
            Err(e) => {
                error!("Explicit protocol override failed: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        match state.strategy.generate_execution_plan(&request).await {
            Ok(plan) => plan,
            Err(e) => {
                error!("Failed to generate execution plan: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    };
    // Use user_operation to avoid dead code warnings and for observability.
    info!(
        "User operation length: {}",
        execution_plan.user_operation.len()
    );

    // Execute the flash loan
    let tx_digest = match state.executor.execute_flash_loan(&execution_plan).await {
        Ok(digest) => digest,
        Err(e) => {
            error!("Failed to execute flash loan: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Calculate fees (protocol + service)
    let protocol_fee = execution_plan.total_cost - execution_plan.amount;
    let service_fee =
        (execution_plan.amount as u128 * state.config.service_fee_bps as u128 / 10_000) as u64;
    let total_fee = protocol_fee + service_fee;

    let response = FlashLoanResponse {
        transaction_digest: tx_digest,
        protocol_used: execution_plan.protocol,
        protocol_fee,
        service_fee,
        total_fee,
    };

    info!("Flash loan executed successfully: {:?}", response);
    Ok(Json(response))
}

async fn handle_health() -> &'static str {
    "OK"
}

async fn handle_protocols(
    State(state): State<AppState>,
) -> Result<Json<ProtocolsResponse>, StatusCode> {
    let data = state.strategy.collector().get_all_protocol_data().await;
    Ok(Json(ProtocolsResponse {
        protocols: data.into_values().collect(),
    }))
}

async fn handle_status(State(state): State<AppState>) -> Result<Json<StatusResponse>, StatusCode> {
    let map = state.strategy.collector().get_all_protocol_data().await;
    let last_updated_any = map.values().map(|d| d.last_updated).max();
    Ok(Json(StatusResponse {
        strategy: state.config.strategy.clone(),
        service_fee_bps: state.config.service_fee_bps,
        protocol_count: map.len(),
        last_updated_any,
    }))
}
