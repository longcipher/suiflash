use artemis::types::Strategy;
use async_trait::async_trait;
use eyre::Result;
use tracing::{debug, info};

use crate::{
    collectors::ProtocolDataCollector,
    config::{Config, FlashLoanRequest, Protocol, ProtocolData},
};

#[derive(Debug, Clone)]
pub struct FlashLoanStrategy {
    config: Config,
    collector: ProtocolDataCollector,
}

impl FlashLoanStrategy {
    pub fn new(config: Config, collector: ProtocolDataCollector) -> Self {
        Self { config, collector }
    }

    pub fn collector(&self) -> &ProtocolDataCollector {
        &self.collector
    }

    /// Find the best protocol for a flash loan request based on strategy
    pub async fn find_best_protocol(&self, request: &FlashLoanRequest) -> Result<Protocol> {
        let protocol_data = self.collector.get_all_protocol_data().await;

        // Filter protocols that have sufficient liquidity
        let viable_protocols: Vec<_> = protocol_data
            .iter()
            .filter(|(_, data)| data.available_liquidity >= request.amount)
            .collect();

        if viable_protocols.is_empty() {
            eyre::bail!(
                "No protocol has sufficient liquidity for amount: {}",
                request.amount
            );
        }

        let best_protocol = match self.config.strategy.as_str() {
            "cheapest" => self.find_cheapest_protocol(&viable_protocols),
            "highest_liquidity" => self.find_highest_liquidity_protocol(&viable_protocols),
            _ => {
                debug!(
                    "Unknown strategy '{}', defaulting to cheapest",
                    self.config.strategy
                );
                self.find_cheapest_protocol(&viable_protocols)
            }
        };

        info!(
            "Selected protocol {:?} for flash loan of {} SUI",
            best_protocol, request.amount
        );
        Ok(best_protocol)
    }

    fn find_cheapest_protocol(&self, protocols: &[(&Protocol, &ProtocolData)]) -> Protocol {
        protocols
            .iter()
            .min_by_key(|(_, data)| data.fee_bps)
            .map(|(protocol, _)| **protocol)
            .unwrap_or(Protocol::Navi) // Default fallback
    }

    fn find_highest_liquidity_protocol(
        &self,
        protocols: &[(&Protocol, &ProtocolData)],
    ) -> Protocol {
        protocols
            .iter()
            .max_by_key(|(_, data)| data.available_liquidity)
            .map(|(protocol, _)| **protocol)
            .unwrap_or(Protocol::Navi) // Default fallback
    }

    /// Calculate total cost for a flash loan including fees
    pub async fn calculate_cost(
        &self,
        request: &FlashLoanRequest,
        protocol: Protocol,
    ) -> Result<u64> {
        let protocol_data = self
            .collector
            .get_protocol_data(protocol)
            .await
            .ok_or_else(|| eyre::eyre!("No data available for protocol {:?}", protocol))?;

        // Protocol fee = amount * fee_bps / 10000
        let protocol_fee = (request.amount as u128 * protocol_data.fee_bps as u128) / 10_000;
        let total_cost = request.amount + protocol_fee as u64;

        debug!(
            "Flash loan cost calculation: amount={}, fee_bps={}, protocol_fee={}, total={}",
            request.amount, protocol_data.fee_bps, protocol_fee, total_cost
        );

        Ok(total_cost)
    }

    /// Generate execution plan for the flash loan
    pub async fn generate_execution_plan(
        &self,
        request: &FlashLoanRequest,
    ) -> Result<ExecutionPlan> {
        let best_protocol = self.find_best_protocol(request).await?;
        let total_cost = self.calculate_cost(request, best_protocol).await?;

        Ok(ExecutionPlan {
            protocol: best_protocol,
            amount: request.amount,
            total_cost,
            user_operation: request.user_operation.clone(),
            callback_recipient: request.callback_recipient.clone(),
            callback_payload: request.callback_payload.clone(),
        })
    }

    pub async fn override_protocol(
        &self,
        request: &FlashLoanRequest,
        protocol: Protocol,
    ) -> Result<ExecutionPlan> {
        // Ensure liquidity
        let data = self
            .collector
            .get_protocol_data(protocol)
            .await
            .ok_or_else(|| eyre::eyre!("No data for protocol {:?}", protocol))?;
        if data.available_liquidity < request.amount {
            eyre::bail!("Protocol {:?} insufficient liquidity", protocol);
        }
        let total_cost = self.calculate_cost(request, protocol).await?;
        Ok(ExecutionPlan {
            protocol,
            amount: request.amount,
            total_cost,
            user_operation: request.user_operation.clone(),
            callback_recipient: request.callback_recipient.clone(),
            callback_payload: request.callback_payload.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub protocol: Protocol,
    pub amount: u64,
    pub total_cost: u64,
    pub user_operation: String, // User's arbitrary operation
    pub callback_recipient: Option<String>,
    pub callback_payload: Option<String>,
}

// Placeholder Event type for Artemis integration
#[allow(dead_code)] // Placeholder event type; retained for future Artemis integration.
#[derive(Debug, Clone)]
pub struct FlashLoanEvent {
    pub request: FlashLoanRequest,
    pub timestamp: u64,
}

// Placeholder implementation for Artemis Strategy interface
#[async_trait]
impl Strategy<FlashLoanEvent, ExecutionPlan> for FlashLoanStrategy {
    async fn sync_state(&mut self) -> Result<()> {
        // Sync state with collectors if needed
        Ok(())
    }

    async fn process_event(&mut self, _event: FlashLoanEvent) -> Vec<ExecutionPlan> {
        // For now, we don't process events directly from Artemis
        // Flash loans are initiated via REST API
        vec![]
    }
}
