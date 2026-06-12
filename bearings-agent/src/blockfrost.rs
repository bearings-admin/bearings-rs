//! Blockfrost API wrapper — read-only Cardano wallet surveillance.
//! API docs: https://docs.blockfrost.io
//! Free tier: 50,000 requests/day — sufficient for hourly monitoring.
//!
//! NOTE FOR GASPAR: This is the Cardano integration layer.
//! Key constraint: we NEVER sign transactions here.
//! We read wallet state only. Private keys never touch this binary.
//!
//! Blockfrost response shapes are documented at docs.blockfrost.io/api
//! The address endpoint returns an array of amounts (one per asset, lovelace first).

use crate::error::AgentError;
use anyhow::{Context, Result};
use serde::Deserialize;

pub struct BlockfrostClient {
    project_id: String,
    base_url: String,
    client: reqwest::Client,
}
// Note: headers() rebuilds HeaderMap on every call.
// This is acceptable — HeaderMap construction is cheap and
// project_id changes only at startup. If profiling shows it
// matters, move to a stored Arc<HeaderMap>.

/// Address info from GET /addresses/{address}
#[derive(Debug, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub amount: Vec<AssetAmount>,
    pub stake_address: Option<String>,
    #[serde(rename = "type")]
    pub address_type: String, // "shelley" or "byron"
    pub managed: Option<bool>,
}

/// An asset amount in a Cardano address
#[derive(Debug, Deserialize)]
pub struct AssetAmount {
    pub unit: String,     // "lovelace" for ADA, or policy+asset hex for native tokens
    pub quantity: String, // String because Cardano amounts can exceed u64 for total supply
}

impl AssetAmount {
    /// Parse quantity as u64. Panics if the string is malformed.
    pub fn quantity_u64(&self) -> u64 {
        self.quantity.parse().unwrap_or(0)
    }
}

/// Transaction summary from GET /addresses/{address}/transactions
#[derive(Debug, Deserialize)]
pub struct AddressTransaction {
    pub tx_hash: String,
    pub tx_index: u32,
    pub block_height: u64,
    pub block_time: i64, // Unix timestamp
}

/// Transaction UTXO detail from GET /txs/{hash}/utxos
#[derive(Debug, Deserialize)]
pub struct TxUtxos {
    pub hash: String,
    pub inputs: Vec<TxUtxo>,
    pub outputs: Vec<TxUtxo>,
}

#[derive(Debug, Deserialize)]
pub struct TxUtxo {
    pub address: String,
    pub amount: Vec<AssetAmount>,
    pub output_index: Option<u32>,
    pub data_hash: Option<String>,
}

impl BlockfrostClient {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            project_id: std::env::var("BLOCKFROST_PROJECT_ID")
                .context("BLOCKFROST_PROJECT_ID not set")?,
            base_url: "https://cardano-mainnet.blockfrost.io/api/v0".to_string(),
            client: reqwest::Client::new(),
        })
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert("project_id", self.project_id.parse().unwrap());
        h
    }

    /// Get full address info including lovelace balance.
    pub async fn address_info(&self, address: &str) -> Result<AddressInfo, AgentError> {
        let url = format!("{}/addresses/{}", self.base_url, address);
        let info: AddressInfo = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?
            .json()
            .await?;
        Ok(info)
    }

    /// Get the ADA balance in lovelace (1 ADA = 1,000,000 lovelace).
    pub async fn wallet_balance_lovelace(&self, address: &str) -> Result<u64, AgentError> {
        let info = self.address_info(address).await?;
        let lovelace = info
            .amount
            .iter()
            .find(|a| a.unit == "lovelace")
            .map(|a| a.quantity_u64())
            .unwrap_or(0);
        Ok(lovelace)
    }

    /// Get the ADA balance as a human-readable f64.
    pub async fn wallet_balance_ada(&self, address: &str) -> Result<f64, AgentError> {
        let lovelace = self.wallet_balance_lovelace(address).await?;
        Ok(lovelace as f64 / 1_000_000.0)
    }

    /// Get recent transactions for an address, newest first.
    pub async fn recent_transactions(
        &self,
        address: &str,
        count: u32,
    ) -> Result<Vec<AddressTransaction>, AgentError> {
        let url = format!(
            "{}/addresses/{}/transactions?order=desc&count={}",
            self.base_url, address, count
        );
        let txs: Vec<AddressTransaction> = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?
            .json()
            .await?;
        Ok(txs)
    }

    /// Get the UTXO detail for a specific transaction.
    /// Used to determine the ADA amount received by our wallet.
    pub async fn tx_utxos(&self, tx_hash: &str) -> Result<TxUtxos, AgentError> {
        let url = format!("{}/txs/{}/utxos", self.base_url, tx_hash);
        let utxos: TxUtxos = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?
            .json()
            .await?;
        Ok(utxos)
    }

    /// Calculate the net ADA received by a specific address in a transaction.
    /// Sums outputs to the address, subtracts inputs from the address.
    pub fn net_received(&self, utxos: &TxUtxos, our_address: &str) -> f64 {
        let received: u64 = utxos
            .outputs
            .iter()
            .filter(|o| o.address == our_address)
            .flat_map(|o| o.amount.iter().filter(|a| a.unit == "lovelace"))
            .map(|a| a.quantity_u64())
            .sum();

        let sent: u64 = utxos
            .inputs
            .iter()
            .filter(|i| i.address == our_address)
            .flat_map(|i| i.amount.iter().filter(|a| a.unit == "lovelace"))
            .map(|a| a.quantity_u64())
            .sum();

        (received as f64 - sent as f64) / 1_000_000.0
    }
}
