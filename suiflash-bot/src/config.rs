use std::fmt;

use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Load configuration from multiple sources with priority:
    /// 1. config.toml file (if exists)
    /// 2. Environment variables (with SUIFLASH_ prefix)
    /// 3. Default values
    ///
    /// # Errors
    ///
    /// Returns an error if required configuration values are missing or invalid
    pub fn load() -> Result<Self, ConfigError> {
        let mut builder = ConfigBuilder::builder()
            // Set default values
            .set_default("sui_rpc_url", "https://fullnode.testnet.sui.io:443")?
            .set_default("server_port", 3000)?
            .set_default("refresh_interval_ms", 10000)?
            .set_default("strategy", "cheapest")?
            .set_default("contract_package_id", "0x1")?
            .set_default("navi_package_id", "0x2")?
            .set_default("bucket_package_id", "0x3")?
            .set_default("scallop_package_id", "0x4")?
            .set_default("service_fee_bps", 40)?;

        // Try to load from config.toml file (optional)
        if std::path::Path::new("config.toml").exists() {
            builder = builder.add_source(File::with_name("config"));
        }

        // Add environment variables with SUIFLASH_ prefix
        // This allows using SUIFLASH_PRIVATE_KEY instead of PRIVATE_KEY
        builder = builder.add_source(
            Environment::with_prefix("SUIFLASH")
                .separator("_")
                .ignore_empty(true),
        );

        // Also support legacy environment variables without prefix for backward compatibility
        builder = builder.add_source(Environment::default().ignore_empty(true));

        let config = builder.build()?;
        config.try_deserialize()
    }

    /// Legacy method for backward compatibility with environment variables
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing
    pub fn from_env() -> eyre::Result<Self> {
        // First try the new config system
        match Self::load() {
            Ok(config) => Ok(config),
            Err(e) => {
                tracing::warn!(
                    "Failed to load config using config crate: {}, falling back to legacy env vars",
                    e
                );
                // Fallback to the old method
                Self::from_env_legacy()
            }
        }
    }

    /// Legacy environment variable loading method
    fn from_env_legacy() -> eyre::Result<Self> {
        dotenv::dotenv().ok();

        Ok(Self {
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

    /// Load configuration from a specific TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn from_toml<P: AsRef<std::path::Path>>(path: P) -> eyre::Result<Self> {
        let builder = ConfigBuilder::builder().add_source(File::from(path.as_ref()));

        let config = builder
            .build()
            .map_err(|e| eyre::eyre!("Failed to build config: {}", e))?;

        config
            .try_deserialize()
            .map_err(|e| eyre::eyre!("Failed to deserialize config: {}", e))
    }

    /// Save configuration to a TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written
    pub fn save_to_toml<P: AsRef<std::path::Path>>(&self, path: P) -> eyre::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| eyre::eyre!("Failed to serialize config: {}", e))?;

        std::fs::write(path, content).map_err(|e| eyre::eyre!("Failed to write config file: {}", e))
    }
}

/// Asset types supported by the flash loan aggregator
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Asset {
    SUI,
    USDC,
    USDT,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SUI => write!(f, "SUI"),
            Self::USDC => write!(f, "USDC"),
            Self::USDT => write!(f, "USDT"),
        }
    }
}

impl Asset {
    #[allow(dead_code)]
    pub const fn to_type_tag(self) -> &'static str {
        match self {
            Self::SUI => "0x2::sui::SUI",
            Self::USDC => {
                "0x2::coin::COIN<0x5d4b302506645c37ff133b98c4b50a5ae14841659738d6d733d59d0d217a93bf::coin::COIN>"
            }
            Self::USDT => {
                "0x2::coin::COIN<0xc060006111016b8a020ad5b33834984a437aaa7d3c74c18e09a95d48aceab08c::coin::COIN>"
            }
        }
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
