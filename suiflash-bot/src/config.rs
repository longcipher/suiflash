use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub sui_rpc_url: String,
    pub private_key: String,
    pub sui_flash_package_id: String,
    pub sui_flash_config_object_id: String,
    pub server_port: u16,
    pub refresh_interval_ms: u64,
    pub strategy: String, // "cheapest" or "highest_liquidity"
    pub contract_package_id: String,
    pub navi_package_id: String,
    pub bucket_package_id: String,
    pub scallop_package_id: String,
    pub service_fee_bps: u64, // off-chain expectation (mirror of on-chain Config)
}

impl Config {
    pub fn from_env() -> eyre::Result<Self> {
        dotenv::dotenv().ok();

        Ok(Config {
            sui_rpc_url: std::env::var("SUI_RPC_URL")
                .unwrap_or_else(|_| "https://fullnode.testnet.sui.io:443".to_string()),
            private_key: std::env::var("PRIVATE_KEY")
                .map_err(|_| eyre::eyre!("PRIVATE_KEY environment variable required"))?,
            sui_flash_package_id: std::env::var("SUI_FLASH_PACKAGE_ID")
                .map_err(|_| eyre::eyre!("SUI_FLASH_PACKAGE_ID environment variable required"))?,
            sui_flash_config_object_id: std::env::var("SUI_FLASH_CONFIG_OBJECT_ID").map_err(
                |_| eyre::eyre!("SUI_FLASH_CONFIG_OBJECT_ID environment variable required"),
            )?,
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            refresh_interval_ms: std::env::var("REFRESH_INTERVAL_MS")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
            strategy: std::env::var("STRATEGY").unwrap_or_else(|_| "cheapest".to_string()),
            contract_package_id: std::env::var("CONTRACT_PACKAGE_ID")
                .unwrap_or_else(|_| "0x1".to_string()),
            navi_package_id: std::env::var("NAVI_PACKAGE_ID").unwrap_or_else(|_| "0x2".to_string()),
            bucket_package_id: std::env::var("BUCKET_PACKAGE_ID")
                .unwrap_or_else(|_| "0x3".to_string()),
            scallop_package_id: std::env::var("SCALLOP_PACKAGE_ID")
                .unwrap_or_else(|_| "0x4".to_string()),
            service_fee_bps: std::env::var("SERVICE_FEE_BPS")
                .unwrap_or_else(|_| "40".to_string()) // default 0.40%
                .parse()
                .unwrap_or(40),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashLoanRequest {
    pub asset: String,
    pub amount: u64,
    pub route_mode: RouteMode,
    pub explicit_protocol: Option<Protocol>,
    pub user_operation: String, // Simplified: user's operation as string
    pub callback_recipient: Option<String>, // hex address of contract handling callback
    pub callback_payload: Option<String>, // base64 or hex encoded payload
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteMode {
    Explicit,
    BestCost,
    BestLiquidity,
}

// Removed unused as_str method to silence dead_code warning.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    Navi = 0,
    Bucket = 1,
    Scallop = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolData {
    pub protocol: Protocol,
    pub fee_bps: u64,
    pub available_liquidity: u64,
    pub last_updated: u64, // timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashLoanResponse {
    pub transaction_digest: String,
    pub protocol_used: Protocol,
    pub protocol_fee: u64,
    pub service_fee: u64,
    pub total_fee: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolsResponse {
    pub protocols: Vec<ProtocolData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub strategy: String,
    pub service_fee_bps: u64,
    pub protocol_count: usize,
    pub last_updated_any: Option<u64>,
}
