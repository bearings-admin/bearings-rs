//! Data access for the public transparency page: operating costs and the
//! "keep the lights on" wallet (USDC on Base).

use crate::db::SupabaseClient;
use crate::error::AppError;
use async_trait::async_trait;
use std::collections::HashMap;

/// Accept numeric-or-string for amount_usd (PostgREST may render `numeric` either way).
fn de_f64<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrStr {
        N(f64),
        S(String),
    }
    Ok(match NumOrStr::deserialize(d)? {
        NumOrStr::N(n) => n,
        NumOrStr::S(s) => s.parse().unwrap_or(0.0),
    })
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OperatingCost {
    pub label: String,
    #[serde(deserialize_with = "de_f64")]
    pub amount_usd: f64,
    pub cadence: String, // monthly | annual | one-time
    pub category: Option<String>,
    pub note: Option<String>,
}

/// Current state of the wallet that funds hosting.
#[derive(Debug, Clone, Default)]
pub struct WalletState {
    pub address: String,
    pub chain: String,
    pub balance_usd: f64,
    pub updated: String,
}

/// A kindred resource we lean on and credit (guide, magazine, archive, directory).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct KindredSource {
    pub name: String,
    pub url: String,
    pub blurb: Option<String>,
    pub language: Option<String>,
    pub kind: Option<String>,
}

/// A public feed our nightly reader consumes — credited on the transparency page.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FeedCredit {
    pub org_name: Option<String>,
    pub url: String,
    pub feed_type: Option<String>,
}

#[async_trait]
pub trait TransparencyRepository: Send + Sync {
    async fn costs(&self) -> Result<Vec<OperatingCost>, AppError>;
    async fn wallet(&self) -> Result<WalletState, AppError>;
    async fn sources(&self) -> Result<Vec<KindredSource>, AppError>;
    async fn feeds(&self) -> Result<Vec<FeedCredit>, AppError>;
}

pub struct SupabaseTransparencyRepository {
    db: SupabaseClient,
}
impl SupabaseTransparencyRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TransparencyRepository for SupabaseTransparencyRepository {
    async fn costs(&self) -> Result<Vec<OperatingCost>, AppError> {
        let url = format!(
            "{}/rest/v1/operating_costs?select=label,amount_usd,cadence,category,note&active=eq.true&order=cadence.asc,label.asc",
            self.db.url
        );
        self.db.get_json::<Vec<OperatingCost>>(&url).await
    }

    async fn wallet(&self) -> Result<WalletState, AppError> {
        let url = format!(
            "{}/rest/v1/platform_settings?select=key,value&key=in.(lights_wallet_address,lights_wallet_chain,lights_wallet_balance_usd,lights_wallet_updated)",
            self.db.url
        );
        #[derive(serde::Deserialize)]
        struct Setting {
            key: String,
            value: Option<String>,
        }
        let rows: Vec<Setting> = self.db.get_json(&url).await?;
        let m: HashMap<String, String> = rows
            .into_iter()
            .filter_map(|s| s.value.map(|v| (s.key, v)))
            .collect();
        Ok(WalletState {
            address: m.get("lights_wallet_address").cloned().unwrap_or_default(),
            chain: m
                .get("lights_wallet_chain")
                .cloned()
                .unwrap_or_else(|| "Base".into()),
            balance_usd: m
                .get("lights_wallet_balance_usd")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0),
            updated: m.get("lights_wallet_updated").cloned().unwrap_or_default(),
        })
    }

    async fn sources(&self) -> Result<Vec<KindredSource>, AppError> {
        let url = format!(
            "{}/rest/v1/kindred_sources?select=name,url,blurb,language,kind&active=eq.true&order=sort_order.asc,name.asc",
            self.db.url
        );
        self.db.get_json::<Vec<KindredSource>>(&url).await
    }

    async fn feeds(&self) -> Result<Vec<FeedCredit>, AppError> {
        let url = format!(
            "{}/rest/v1/watched_feeds?select=org_name,url,feed_type&active=eq.true&order=org_name.asc",
            self.db.url
        );
        self.db.get_json::<Vec<FeedCredit>>(&url).await
    }
}
