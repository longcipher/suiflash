mod collectors;
mod config;
mod executors;
mod strategies;
mod navi_ptb_builder;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod api_tests;

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use clap::Parser;
use collectors::ProtocolFlashLoanCollector;
use config::{Config, FlashLoanRequest, FlashLoanResponse, ProtocolsResponse, StatusResponse};
use executors::FlashLoanExecutor;
use eyre::Result;
use strategies::FlashLoanStrategy;
use tokio::net::TcpListener;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "suiflash_bot")]
#[command(about = "SuiFlash Bot - Capital-light multi-protocol flash loan aggregator")]
struct Args {
    /// Path to configuration file
    #[arg(short = 'c', long = "config", help = "Configuration file path")]
    config: Option<std::path::PathBuf>,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub strategy: FlashLoanStrategy,
    pub executor: FlashLoanExecutor,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt().init();

    // Load configuration
    let config = if let Some(config_path) = args.config {
        Config::load_from_file(config_path)?
    } else {
        Config::load()?
    };
    info!("Starting SuiFlash Bot with config: {:?}", config);

    // Check if we should just test the collectors
    if std::env::var("TEST_COLLECTORS").is_ok() {
        info!("Running collector test mode");
        return test_collectors(config).await;
    }

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
    let collector = ProtocolFlashLoanCollector::new(config.clone()).await;
    let strategy = FlashLoanStrategy::new(config.clone(), collector.clone());
    let executor = FlashLoanExecutor::new(config.clone()).await?;

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

    // Build the router with debug logging
    let app = Router::new()
        .route("/", get(|| async {
            println!("DEBUG: Root endpoint called - printing to stdout");
            eprintln!("DEBUG: Root endpoint called - printing to stderr");
            info!("Root endpoint called");
            "SuiFlash Bot API"
        }))
        .route("/flashloan", post(handle_flash_loan))
        .route("/protocols", get(handle_protocols))
        .route("/status", get(handle_status))
        .route("/health", get(handle_health))
        .with_state(app_state);

    // Start the server
    let addr = format!("127.0.0.1:{}", config.server_port);  // Try localhost instead of 0.0.0.0
    info!("Starting server on {}", addr);

    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => {
            info!("Successfully bound to {}", addr);
            listener
        }
        Err(e) => {
            error!("Failed to bind to {}: {}", addr, e);
            return Err(e.into());
        }
    };

    info!("Server listening and ready to accept connections");
    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
        return Err(e.into());
    }

    // Clean up background tasks
    collector_handle.abort();

    Ok(())
}

/// Handle flash loan requests
///
/// # Errors
///
/// Returns `StatusCode::INTERNAL_SERVER_ERROR` if:
/// - Data collection fails
/// - Execution plan generation fails  
/// - Flash loan execution fails
/// - Service fee calculation overflows
pub async fn handle_flash_loan(
    State(state): State<AppState>,
    Json(mut request): Json<FlashLoanRequest>,
) -> Result<Json<FlashLoanResponse>, StatusCode> {
    info!("Received flash loan request: {:?}", request);
    info!("Current strategy mode: {}", state.config.strategy);

    // Provide default callback recipient if not specified
    if request.callback_recipient.is_none() {
        request.callback_recipient = Some(state.executor.create_dummy_callback_recipient());
        info!(
            "Using default callback recipient: {:?}",
            request.callback_recipient
        );
    }

    // Provide default user operation if empty
    if request.user_operation.is_empty() {
        request.user_operation = "default_noop_operation".to_string();
        info!("Using default user operation: {}", request.user_operation);
    }

    // Determine protocol if explicit routing requested
    let execution_plan = if let Some(p) = request.explicit_protocol {
        match state.strategy.override_protocol(&request, p).await {
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
    let service_fee = u64::try_from(
        u128::from(execution_plan.amount) * u128::from(state.config.service_fee_bps) / 10_000,
    )
    .map_err(|_| {
        error!("Service fee calculation overflow");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
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

pub async fn handle_health() -> &'static str {
    println!("DEBUG: Health endpoint called - printing to stdout");
    eprintln!("DEBUG: Health endpoint called - printing to stderr");
    info!("Health endpoint called");
    "OK"
}

/// Get available protocols and their data
///
/// # Errors
///
/// Returns `StatusCode::INTERNAL_SERVER_ERROR` if data collection fails
pub async fn handle_protocols(
    State(state): State<AppState>,
) -> Result<Json<ProtocolsResponse>, StatusCode> {
    let data = state.strategy.collector().get_all_protocol_data().await;
    Ok(Json(ProtocolsResponse {
        protocols: data.into_values().collect(),
    }))
}

/// Get service status information
///
/// # Errors
///
/// Returns `StatusCode::INTERNAL_SERVER_ERROR` if data collection fails
pub async fn handle_status(
    State(state): State<AppState>,
) -> Result<Json<StatusResponse>, StatusCode> {
    info!("Status endpoint called");
    let map = state.strategy.collector().get_all_protocol_data().await;
    let last_updated_any = map.values().map(|d| d.last_updated).max();
    Ok(Json(StatusResponse {
        strategy: state.config.strategy.clone(),
        service_fee_bps: state.config.service_fee_bps,
        protocol_count: map.len(),
        last_updated_any,
    }))
}

/// Test collector functionality
async fn test_collectors(config: Config) -> Result<()> {
    println!("\n🔍 Testing Protocol Flash Loan Collectors");
    println!("==========================================");

    // Initialize collector
    let collector = ProtocolFlashLoanCollector::new(config.clone()).await;

    // Update all fees - this will fetch and print Navi details
    println!("\n📊 Updating all protocol fees...\n");
    collector.update_all_fees().await?;

    // Print summary
    let all_fees = collector.get_all_flash_loan_fees().await;
    println!("\n📋 Summary - All Protocol Flash Loan Fees:");
    println!("============================================");
    for (protocol, fee_bps) in &all_fees {
        println!("Protocol: {protocol:?} -> Fee: {fee_bps} bps");
    }

    println!("\n✅ Collector test completed successfully!");
    Ok(())
}
