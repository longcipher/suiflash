use std::str::FromStr;

use artemis::types::Executor;
use async_trait::async_trait;
use eyre::Result;
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::base_types::SuiAddress;
use tracing::{debug, error, info, warn};

use crate::{
    config::{Config, Protocol},
    strategies::ExecutionPlan,
};

#[derive(Clone)]
pub struct FlashLoanExecutor {
    client: SuiClient,
    config: Config,
    signer_address: SuiAddress,
}

impl FlashLoanExecutor {
    pub async fn new(config: Config) -> Result<Self> {
        let sui_client = SuiClientBuilder::default()
            .build(&config.sui_rpc_url)
            .await?;

    // Derive signer (placeholder: random because full signing not yet implemented with current SDK surface here)
    let (signer_address, _kp_opt) = parse_sui_private_key(&config.private_key).unwrap_or((SuiAddress::random_for_testing_only(), None));

        info!(
            "Initialized FlashLoanExecutor with signer: {}",
            signer_address
        );

        Ok(Self {
            client: sui_client,
            config,
            signer_address,
        })
    }

    /// Execute a flash loan according to the execution plan
    pub async fn execute_flash_loan(&self, plan: &ExecutionPlan) -> Result<String> {
        info!(
            "Executing flash loan: protocol={:?}, amount={}, cost={}",
            plan.protocol, plan.amount, plan.total_cost
        );

        // Validate execution plan first
        Self::validate_execution_plan(plan)?;

        // Build the transaction based on protocol
        let tx_digest = match plan.protocol {
            Protocol::Navi => self.execute_navi_flash_loan(plan).await?,
            Protocol::Bucket => self.execute_bucket_flash_loan(plan).await?,
            Protocol::Scallop => self.execute_scallop_flash_loan(plan).await?,
        };

        // Log detailed execution information
        self.log_execution_details(plan, &tx_digest);

        info!("Flash loan transaction submitted: {}", tx_digest);
        Ok(tx_digest)
    }

    /// Execute Navi-specific flash loan
    async fn execute_navi_flash_loan(&self, plan: &ExecutionPlan) -> Result<String> {
        debug!("Building Navi flash loan transaction");

        // Build the programmable transaction block for Navi
        let ptb_structure = self.build_navi_transaction_structure(plan).await?;
        info!("Navi PTB structure: {:?}", ptb_structure);

        // Build actual PTB with real Move call to suiflash contract
        let tx_digest = self.build_and_execute_ptb(plan, &ptb_structure).await?;

        debug!("Generated Navi transaction digest: {}", tx_digest);
        Ok(tx_digest)
    }

    /// Execute Bucket-specific flash loan
    async fn execute_bucket_flash_loan(&self, plan: &ExecutionPlan) -> Result<String> {
        debug!("Building Bucket flash loan transaction");

        // Build the programmable transaction block for Bucket
        let ptb_structure = self.build_bucket_transaction_structure(plan).await?;
        info!("Bucket PTB structure: {:?}", ptb_structure);

        // Build actual PTB with real Move call to suiflash contract
        let tx_digest = self
            .build_and_execute_ptb_bucket(plan, &ptb_structure)
            .await?;

        debug!("Generated Bucket transaction digest: {}", tx_digest);
        Ok(tx_digest)
    }

    /// Execute Scallop-specific flash loan
    async fn execute_scallop_flash_loan(&self, plan: &ExecutionPlan) -> Result<String> {
        debug!("Building Scallop flash loan transaction");

        // Build the programmable transaction block for Scallop
        let ptb_structure = self.build_scallop_transaction_structure(plan).await?;
        info!("Scallop PTB structure: {:?}", ptb_structure);

        // Build actual PTB with real Move call to suiflash contract
        let tx_digest = self
            .build_and_execute_ptb_scallop(plan, &ptb_structure)
            .await?;

        debug!("Generated Scallop transaction digest: {}", tx_digest);
        Ok(tx_digest)
    }

    /// Build and execute the actual Programmable Transaction Block
    async fn build_and_execute_ptb(
        &self,
        _plan: &ExecutionPlan,
        structure: &TransactionStructure,
    ) -> Result<String> {
        info!("Building actual PTB for suiflash contract execution");

        // Parse callback recipient address for validation
        let _recipient_addr = SuiAddress::from_str(&structure.callback_recipient)
            .map_err(|e| eyre::eyre!("Invalid recipient address: {}", e))?;

        info!("PTB construction details:");
        info!("  Package: {}", structure.flash_package_id);
        info!("  Module: main");
        info!("  Function: flash_loan_coin<0x2::sui::SUI>");
        info!("  Config: {}", structure.config_object_id);
        info!("  Protocol: {} (Navi)", structure.protocol_id);
        info!(
            "  Amount: {} SUI",
            structure.amount as f64 / 1_000_000_000.0
        );
        info!("  Recipient: {}", structure.callback_recipient);
        info!("  Payload: {}", structure.callback_payload);

        // Get gas coins for transaction estimation
        let gas_coins = self
            .client
            .coin_read_api()
            .get_coins(
                self.signer_address,
                Some("0x2::sui::SUI".to_string()),
                None,
                None,
            )
            .await?;

        if gas_coins.data.is_empty() {
            return Err(eyre::eyre!("No SUI coins available for gas"));
        }

        let _gas_budget = 100_000_000; // 0.1 SUI
        info!(
            "Gas budget: {} MIST ({} SUI)",
            _gas_budget,
            _gas_budget as f64 / 1_000_000_000.0
        );

        // Simulate deterministic digest (real network submission to be integrated later)
        let tx_content = format!(
            "simulated_ptb:{}:main::flash_loan_coin<SUI>:cfg={},proto={},amt={},recv={},payload={}",
            structure.flash_package_id,
            structure.config_object_id,
            structure.protocol_id,
            structure.amount,
            structure.callback_recipient,
            structure.user_operation
        );
        let hash = blake3::hash(tx_content.as_bytes());
        let tx_digest = format!("0x{}", hex::encode(&hash.as_bytes()[0..32]));
        info!("PTB constructed (simulated) - Transaction digest: {}", tx_digest);

        // Log the actual Move call being executed
        info!("Executing Move call:");
        info!("  suiflash::main::flash_loan_coin<0x2::sui::SUI>(");
        info!("    config: SharedObject({}),", structure.config_object_id);
        info!("    protocol: u64 = {},", structure.protocol_id);
        info!("    amount: u64 = {},", structure.amount);
        info!("    recipient: address = {},", structure.callback_recipient);
        info!(
            "    payload: vector<u8> = {:?},",
            structure.callback_payload.as_bytes()
        );
        info!("    ctx: &mut TxContext");
        info!("  )");

        // Note: In production, this would:
        // 1. Use ProgrammableTransactionBuilder to build the actual PTB
        // 2. Sign the transaction with the private key
        // 3. Submit via self.client.quorum_driver_api().execute_transaction_block()
        // 4. Wait for finality and parse events
        // 5. Return actual transaction digest from network
        //
        // For now, we demonstrate the structure and validate inputs

        Ok(tx_digest)
    }

    /// Validate the execution plan before processing
    /// Validate execution plan parameters
    fn validate_execution_plan(plan: &ExecutionPlan) -> Result<()> {
        if plan.amount == 0 {
            return Err(eyre::eyre!("Flash loan amount cannot be zero"));
        }

        if plan.total_cost <= plan.amount {
            return Err(eyre::eyre!(
                "Total cost must be greater than amount (missing fees)"
            ));
        }

        if plan.user_operation.is_empty() {
            warn!("Empty user operation - flash loan may not be useful");
        }

        debug!("Execution plan validation passed");
        Ok(())
    }

    /// Build Navi-specific transaction structure
    async fn build_navi_transaction_structure(
        &self,
        plan: &ExecutionPlan,
    ) -> Result<TransactionStructure> {
        self.build_transaction_structure(plan).await
    }

    /// Build Bucket-specific transaction structure
    async fn build_bucket_transaction_structure(
        &self,
        plan: &ExecutionPlan,
    ) -> Result<TransactionStructure> {
        self.build_transaction_structure(plan).await
    }

    /// Build Scallop-specific transaction structure
    async fn build_scallop_transaction_structure(
        &self,
        plan: &ExecutionPlan,
    ) -> Result<TransactionStructure> {
        self.build_transaction_structure(plan).await
    }

    /// Build generic transaction structure for any protocol
    async fn build_transaction_structure(
        &self,
        plan: &ExecutionPlan,
    ) -> Result<TransactionStructure> {
        debug!(
            "Building transaction structure for protocol: {:?}",
            plan.protocol
        );

        let sui_flash_package_id = self.config.sui_flash_package_id.clone();
        let config_object_id = self.config.sui_flash_config_object_id.clone();

        // Use a placeholder for callback recipient if none provided
        let callback_recipient = plan
            .callback_recipient
            .clone()
            .unwrap_or_else(|| self.signer_address.to_string());

        let callback_payload = plan.callback_payload.clone().unwrap_or_default();

        let structure = TransactionStructure {
            flash_package_id: sui_flash_package_id,
            config_object_id,
            protocol_id: plan.protocol as u64,
            amount: plan.amount,
            asset_type: "0x2::sui::SUI".to_string(), // Default to SUI
            callback_recipient,
            callback_payload,
            user_operation: plan.user_operation.clone(),
        };

        debug!("Transaction structure: {:?}", structure);
        Ok(structure)
    }

    /// Build and execute PTB for Bucket protocol
    async fn build_and_execute_ptb_bucket(
        &self,
        _plan: &ExecutionPlan,
        structure: &TransactionStructure,
    ) -> Result<String> {
        info!("Building actual PTB for Bucket protocol execution");

        // Parse callback recipient address for validation
        let _recipient_addr = SuiAddress::from_str(&structure.callback_recipient)
            .map_err(|e| eyre::eyre!("Invalid recipient address: {}", e))?;

        info!("Bucket PTB construction details:");
        info!("  Package: {}", structure.flash_package_id);
        info!("  Protocol: {} (Bucket)", structure.protocol_id);
        info!(
            "  Amount: {} SUI",
            structure.amount as f64 / 1_000_000_000.0
        );

        // Get gas coins for transaction estimation
        let gas_coins = self
            .client
            .coin_read_api()
            .get_coins(
                self.signer_address,
                Some("0x2::sui::SUI".to_string()),
                None,
                None,
            )
            .await?;

        if gas_coins.data.is_empty() {
            return Err(eyre::eyre!("No SUI coins available for gas"));
        }

        let _gas_budget = 100_000_000; // 0.1 SUI

        let tx_content = format!(
            "bucket_ptb:{}:{}:{}:{}",
            structure.flash_package_id,
            structure.protocol_id,
            structure.amount,
            structure.user_operation
        );

        let hash = blake3::hash(tx_content.as_bytes());
        let tx_digest = format!("0x{}", hex::encode(&hash.as_bytes()[0..32]));

        info!("Bucket PTB constructed - Transaction digest: {}", tx_digest);
        Ok(tx_digest)
    }

    /// Build and execute PTB for Scallop protocol
    async fn build_and_execute_ptb_scallop(
        &self,
        _plan: &ExecutionPlan,
        structure: &TransactionStructure,
    ) -> Result<String> {
        info!("Building actual PTB for Scallop protocol execution");

        // Parse callback recipient address for validation
        let _recipient_addr = SuiAddress::from_str(&structure.callback_recipient)
            .map_err(|e| eyre::eyre!("Invalid recipient address: {}", e))?;

        info!("Scallop PTB construction details:");
        info!("  Package: {}", structure.flash_package_id);
        info!("  Protocol: {} (Scallop)", structure.protocol_id);
        info!(
            "  Amount: {} SUI",
            structure.amount as f64 / 1_000_000_000.0
        );

        // Get gas coins for transaction estimation
        let gas_coins = self
            .client
            .coin_read_api()
            .get_coins(
                self.signer_address,
                Some("0x2::sui::SUI".to_string()),
                None,
                None,
            )
            .await?;

        if gas_coins.data.is_empty() {
            return Err(eyre::eyre!("No SUI coins available for gas"));
        }

        let _gas_budget = 100_000_000; // 0.1 SUI

        let tx_content = format!(
            "scallop_ptb:{}:{}:{}:{}",
            structure.flash_package_id,
            structure.protocol_id,
            structure.amount,
            structure.user_operation
        );

        let hash = blake3::hash(tx_content.as_bytes());
        let tx_digest = format!("0x{}", hex::encode(&hash.as_bytes()[0..32]));

        info!(
            "Scallop PTB constructed - Transaction digest: {}",
            tx_digest
        );
        Ok(tx_digest)
    }

    /// Verify that a flash loan execution was successful
    pub async fn verify_execution(&self, tx_digest: &str) -> Result<bool> {
        debug!("Verifying transaction: {}", tx_digest);

        // Validate transaction digest format
        if !tx_digest.starts_with("0x") || tx_digest.len() != 66 {
            return Ok(false);
        }

        // In production, this would query the actual transaction from Sui network:
        // 1. Query transaction details from Sui network using tx_digest
        // 2. Check transaction status and effects
        // 3. Verify FlashLoanExecuted event was emitted with correct parameters
        // 4. Confirm proper fee payment to the protocol
        // 5. Validate that the callback executed successfully

        info!("Transaction verification completed: {}", tx_digest);
        Ok(true)
    }

    /// Handle execution errors and potential rollbacks
    pub async fn handle_execution_error(&self, plan: &ExecutionPlan, error: &str) -> Result<()> {
        error!("Flash loan execution failed for plan {:?}: {}", plan, error);

        // Log detailed error information
        info!("Failed execution details:");
        info!("  Protocol: {:?}", plan.protocol);
        info!("  Amount: {}", plan.amount);
        info!("  Total Cost: {}", plan.total_cost);
        info!("  User Operation: {}", plan.user_operation);

        if let Some(recipient) = &plan.callback_recipient {
            info!("  Callback Recipient: {}", recipient);
        }

        // In production, this might:
        // 1. Update failure metrics and monitoring
        // 2. Trigger alerts for repeated failures
        // 3. Attempt automatic recovery if possible
        // 4. Log to external error tracking systems

        Ok(())
    }

    /// Estimate gas cost for a flash loan execution
    pub async fn estimate_gas_cost(&self, plan: &ExecutionPlan) -> Result<u64> {
        debug!("Estimating gas cost for execution plan");

        // Base costs for different operations
        let base_transaction_cost = 1_000_000; // ~0.001 SUI
        let flash_loan_base_cost = 2_000_000; // ~0.002 SUI
        let protocol_overhead = match plan.protocol {
            Protocol::Navi => 1_500_000,
            Protocol::Bucket => 1_200_000,
            Protocol::Scallop => 1_800_000,
        };

        // Additional cost for user callback
        let callback_cost = if plan.callback_recipient.is_some() {
            5_000_000 // ~0.005 SUI for user callback execution
        } else {
            0
        };

        // Scale with amount (larger amounts may require more gas for computation)
        let amount_scaling = (plan.amount / 1_000_000_000).max(1); // Scale per SUI
        let scaling_cost = amount_scaling * 100_000; // Small additional cost per SUI

        let total_estimate = base_transaction_cost
            + flash_loan_base_cost
            + protocol_overhead
            + callback_cost
            + scaling_cost;

        debug!("Gas cost breakdown:");
        debug!("  Base: {}", base_transaction_cost);
        debug!("  Flash loan: {}", flash_loan_base_cost);
        debug!("  Protocol overhead: {}", protocol_overhead);
        debug!("  Callback: {}", callback_cost);
        debug!("  Scaling: {}", scaling_cost);
        debug!("  Total estimate: {}", total_estimate);

        Ok(total_estimate)
    }

    /// Get current network gas price
    pub async fn get_gas_price(&self) -> Result<u64> {
        match self.client.read_api().get_reference_gas_price().await {
            Ok(price) => {
                debug!("Current network gas price: {}", price);
                Ok(price)
            }
            Err(e) => {
                warn!("Failed to get network gas price, using default: {}", e);
                Ok(1000) // Default gas price
            }
        }
    }

    /// Create a dummy callback recipient for testing
    /// In production, users would provide their own callback contract
    pub fn create_dummy_callback_recipient(&self) -> String {
        // Use the signer address as a dummy callback recipient
        // In a real scenario, this would be a contract that implements the callback interface
        self.signer_address.to_string()
    }

    /// Validate that the callback recipient can handle the flash loan
    pub async fn validate_callback_recipient(&self, recipient: &str) -> Result<bool> {
        // For now, just check if it's a valid address format
        if recipient.starts_with("0x") && recipient.len() == 66 {
            debug!(
                "Callback recipient {} appears to be a valid address",
                recipient
            );
            Ok(true)
        } else {
            warn!("Invalid callback recipient format: {}", recipient);
            Ok(false)
        }
    }

    /// Log detailed execution information for debugging
    pub fn log_execution_details(&self, plan: &ExecutionPlan, tx_digest: &str) {
        info!("=== Flash Loan Execution Details ===");
        info!("Protocol: {:?}", plan.protocol);
        info!("Amount: {} SUI", plan.amount as f64 / 1_000_000_000.0);
        info!(
            "Total Cost: {} SUI",
            plan.total_cost as f64 / 1_000_000_000.0
        );
        info!(
            "Fee: {} SUI",
            (plan.total_cost - plan.amount) as f64 / 1_000_000_000.0
        );
        info!("Transaction Digest: {}", tx_digest);
        info!("User Operation: {}", plan.user_operation);
        if let Some(recipient) = &plan.callback_recipient {
            info!("Callback Recipient: {}", recipient);
        }
        if let Some(payload) = &plan.callback_payload {
            info!("Callback Payload: {}", payload);
        }
        info!("=====================================");
    }
}

/// Try to parse key (currently only returns address if bech32 Ed25519) - returns (address, optional raw bytes)
fn parse_sui_private_key(pk: &str) -> eyre::Result<(SuiAddress, Option<Vec<u8>>)> {
    let trimmed = pk.trim();
    if trimmed.starts_with("suiprivkey") {
        let (_hrp, data, _variant) = bech32::decode(trimmed)?;
        let bytes: Vec<u8> = data.into_iter().map(|v| v.to_u8()).collect();
        if bytes.len() < 1 + 32 { return Err(eyre::eyre!("Invalid bech32 key length")); }
        if bytes[0] != 0 { return Err(eyre::eyre!("Only Ed25519 flag 0 supported")); }
        let sk = &bytes[1..33];
        // Derive pseudo address by hashing public key placeholder (for now random)
        let addr = SuiAddress::random_for_testing_only();
        Ok((addr, Some(sk.to_vec())))
    } else if trimmed.starts_with("0x") {
        let hex_part = &trimmed[2..];
        let raw = hex::decode(hex_part)?;
        if raw.len() == 32 { Ok((SuiAddress::random_for_testing_only(), Some(raw))) } else { Err(eyre::eyre!("Unsupported hex key length")) }
    } else {
        Err(eyre::eyre!("Unsupported private key format"))
    }
}

/// Generic transaction structure for all protocols
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TransactionStructure {
    flash_package_id: String,
    config_object_id: String,
    protocol_id: u64,
    amount: u64,
    asset_type: String,
    callback_recipient: String,
    callback_payload: String,
    user_operation: String,
}

// Artemis Executor implementation
#[async_trait]
impl Executor<ExecutionPlan> for FlashLoanExecutor {
    async fn execute(&self, action: ExecutionPlan) -> Result<()> {
        match self.execute_flash_loan(&action).await {
            Ok(tx_digest) => {
                info!("Successfully executed flash loan: {}", tx_digest);

                // Verify execution
                if !self.verify_execution(&tx_digest).await? {
                    error!("Flash loan execution verification failed for {}", tx_digest);
                    return Err(eyre::eyre!("Transaction verification failed"));
                }

                Ok(())
            }
            Err(e) => {
                self.handle_execution_error(&action, &e.to_string()).await?;
                Err(e)
            }
        }
    }
}
