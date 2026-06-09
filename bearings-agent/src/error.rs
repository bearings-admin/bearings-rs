
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Blockfrost error: {0}")]
    Blockfrost(String),
    #[error("Supabase error: {0}")]
    Supabase(String),
    #[error("Config error: {0}")]
    Config(#[from] anyhow::Error),
}
