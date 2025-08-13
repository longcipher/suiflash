use std::{collections::HashMap, sync::Arc};

use artemis::types::{Collector, CollectorStream};
use async_trait::async_trait;
use eyre::Result;
use tokio::{
    sync::RwLock,
    time::{Duration, interval},
};
use tracing::{debug, error, info};

use crate::config::{Config, Protocol, ProtocolData};

#[derive(Debug, Clone)]
pub struct ProtocolDataCollector {
    config: Config,
    data: Arc<RwLock<HashMap<Protocol, ProtocolData>>>,
}

impl ProtocolDataCollector {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_protocol_data(&self, protocol: Protocol) -> Option<ProtocolData> {
        self.data.read().await.get(&protocol).cloned()
    }

    pub async fn get_all_protocol_data(&self) -> HashMap<Protocol, ProtocolData> {
        self.data.read().await.clone()
    }

    async fn fetch_protocol_data(&self, protocol: Protocol) -> Result<ProtocolData> {
        // Placeholder: In production, this would query the actual protocol contracts
        // using sui-sdk to get real liquidity and fee data

        let (fee_bps, liquidity) = match protocol {
            Protocol::Navi => (8, 10_000_000),   // 0.08%, 10M SUI
            Protocol::Bucket => (5, 5_000_000),  // 0.05%, 5M SUI
            Protocol::Scallop => (9, 8_000_000), // 0.09%, 8M SUI
        };

        Ok(ProtocolData {
            protocol,
            fee_bps,
            available_liquidity: liquidity,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        })
    }

    pub async fn collect_all_data(&self) -> Result<()> {
        info!("Collecting protocol data...");

        let protocols = [Protocol::Navi, Protocol::Bucket, Protocol::Scallop];
        let mut new_data = HashMap::new();

        for protocol in protocols {
            match self.fetch_protocol_data(protocol).await {
                Ok(data) => {
                    debug!(
                        "Updated data for {:?}: fee_bps={}, liquidity={}",
                        protocol, data.fee_bps, data.available_liquidity
                    );
                    new_data.insert(protocol, data);
                }
                Err(e) => {
                    error!("Failed to fetch data for {:?}: {}", protocol, e);
                }
            }
        }

        *self.data.write().await = new_data;
        info!("Protocol data collection complete");

        Ok(())
    }

    pub async fn start_background_collection(&self) {
        let interval_duration = Duration::from_millis(self.config.refresh_interval_ms);
        let mut ticker = interval(interval_duration);

        loop {
            ticker.tick().await;
            if let Err(e) = self.collect_all_data().await {
                error!("Background collection failed: {}", e);
            }
        }
    }
}

// Artemis Collector implementation (placeholder structure)
#[async_trait]
impl Collector<ProtocolData> for ProtocolDataCollector {
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, ProtocolData>> {
        // This would be implemented to provide a stream of protocol data updates
        // For now, returning a placeholder
        todo!("Implement Artemis collector stream")
    }
}
