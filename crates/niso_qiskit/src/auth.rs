//! IBM Quantum authentication
//!
//! Gantree: L10_Qiskit → Auth
//!
//! Supports multiple credential sources (in priority order):
//! 1. Environment variables (IBM_QUANTUM_TOKEN, IBMQ_TOKEN, QISKIT_IBM_TOKEN)
//! 2. Qiskit config file (~/.qiskit/qiskit-ibm.json)
//!
//! Supports two authentication methods:
//! - Direct token (IQP tokens): Used as Bearer token directly
//! - API Key (IBM Cloud): Exchanged for IAM access token first

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

/// Authentication errors
#[derive(Debug, Error)]
pub enum AuthError {
    /// Missing API token
    #[error("IBM Quantum API token not provided. Set IBM_QUANTUM_TOKEN env var or save credentials with Qiskit")]
    MissingToken,

    /// Invalid token format
    #[error("Invalid API token format")]
    InvalidTokenFormat,

    /// Token expired
    #[error("API token has expired")]
    TokenExpired,

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    /// Config file read error
    #[error("Failed to read Qiskit config file: {0}")]
    ConfigFileError(String),

    /// Config file parse error
    #[error("Failed to parse Qiskit config file: {0}")]
    ConfigParseError(String),

    /// IAM token exchange error
    #[error("IAM token exchange failed: {0}")]
    IamTokenExchangeFailed(String),

    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    HttpError(String),
}

/// Qiskit config file entry (matches ~/.qiskit/qiskit-ibm.json format)
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct QiskitConfigEntry {
    /// API token
    token: String,

    /// Channel type
    #[serde(default)]
    channel: Option<String>,

    /// URL
    #[serde(default)]
    url: Option<String>,

    /// Instance/CRN
    #[serde(default)]
    instance: Option<String>,

    /// Plans preference
    #[serde(default)]
    #[allow(dead_code)]
    plans_preference: Option<Vec<String>>,
}

/// Type of API token/key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Direct IQP token (used as Bearer directly)
    IqpToken,
    /// IBM Cloud API Key (needs IAM exchange)
    ApiKey,
}

impl TokenType {
    /// Detect token type from format
    pub fn detect(token: &str) -> Self {
        if token.starts_with("ApiKey-") || token.starts_with("apikey-") {
            TokenType::ApiKey
        } else {
            TokenType::IqpToken
        }
    }
}

/// IAM token response from IBM Cloud
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct IamTokenResponse {
    access_token: String,
    #[serde(default)]
    expires_in: u64, // seconds until expiry
    #[serde(default)]
    #[allow(dead_code)]
    token_type: String,
}

/// Cached IAM access token
#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

/// IBM Quantum credentials
#[derive(Debug, Clone)]
pub struct IbmCredentials {
    /// API token or API key
    api_token: String,

    /// Token type
    token_type: TokenType,

    /// Instance (hub/group/project) or CRN for IBM Cloud
    instance: Option<String>,

    /// Channel (ibm_quantum or ibm_cloud)
    channel: IbmChannel,

    /// Service CRN (Cloud Resource Name) for IBM Cloud
    service_crn: Option<String>,

    /// Cached IAM access token (for API keys)
    cached_iam_token: Arc<RwLock<Option<CachedToken>>>,
}

/// IBM Quantum channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IbmChannel {
    /// IBM Quantum (legacy)
    #[default]
    IbmQuantum,

    /// IBM Cloud
    IbmCloud,
}

impl IbmChannel {
    /// Get base URL for channel (updated for 2025 API)
    pub fn base_url(&self) -> &'static str {
        match self {
            // New API endpoint (2025)
            IbmChannel::IbmQuantum => "https://quantum.cloud.ibm.com",
            IbmChannel::IbmCloud => "https://quantum.cloud.ibm.com",
        }
    }

    /// Get API URL (with /api/v1 prefix)
    pub fn api_url(&self) -> String {
        format!("{}/api/v1", self.base_url())
    }

    /// Get runtime URL (legacy, now uses api_url)
    pub fn runtime_url(&self) -> String {
        self.api_url()
    }
}

/// IAM token endpoint
const IAM_TOKEN_URL: &str = "https://iam.cloud.ibm.com/identity/token";

impl IbmCredentials {
    /// Create new credentials with API token or API key
    pub fn new(api_token: impl Into<String>) -> Self {
        let token = api_token.into();
        let token_type = TokenType::detect(&token);
        Self {
            api_token: token,
            token_type,
            instance: None,
            channel: IbmChannel::default(),
            service_crn: None,
            cached_iam_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Create from environment variable
    pub fn from_env() -> Result<Self, AuthError> {
        let token = std::env::var("IBM_QUANTUM_TOKEN")
            .or_else(|_| std::env::var("IBMQ_TOKEN"))
            .or_else(|_| std::env::var("QISKIT_IBM_TOKEN"))
            .or_else(|_| std::env::var("IBM_CLOUD_API_KEY"))
            .map_err(|_| AuthError::MissingToken)?;

        let instance = std::env::var("IBM_QUANTUM_INSTANCE").ok();
        let service_crn = std::env::var("IBM_QUANTUM_CRN")
            .or_else(|_| std::env::var("SERVICE_CRN"))
            .ok();
        let channel = std::env::var("IBM_QUANTUM_CHANNEL")
            .map(|c| {
                if c.to_lowercase().contains("cloud") {
                    IbmChannel::IbmCloud
                } else {
                    IbmChannel::IbmQuantum
                }
            })
            .unwrap_or_default();

        let token_type = TokenType::detect(&token);

        Ok(Self {
            api_token: token,
            token_type,
            instance,
            channel,
            service_crn,
            cached_iam_token: Arc::new(RwLock::new(None)),
        })
    }

    /// Get the path to the Qiskit config file (~/.qiskit/qiskit-ibm.json)
    fn qiskit_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".qiskit").join("qiskit-ibm.json"))
    }

    /// Load credentials from Qiskit config file (~/.qiskit/qiskit-ibm.json)
    ///
    /// This reads the same config file that Python's QiskitRuntimeService uses.
    /// If `name` is provided, loads that specific named credential set.
    /// Otherwise, loads the default (first entry or one starting with "default").
    pub fn from_qiskit_config(name: Option<&str>) -> Result<Self, AuthError> {
        let config_path = Self::qiskit_config_path().ok_or_else(|| {
            AuthError::ConfigFileError("Could not determine home directory".into())
        })?;

        if !config_path.exists() {
            return Err(AuthError::ConfigFileError(format!(
                "Qiskit config file not found: {}",
                config_path.display()
            )));
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| AuthError::ConfigFileError(format!("{}: {}", config_path.display(), e)))?;

        let config: HashMap<String, QiskitConfigEntry> = serde_json::from_str(&content)
            .map_err(|e| AuthError::ConfigParseError(e.to_string()))?;

        if config.is_empty() {
            return Err(AuthError::ConfigParseError(
                "No credentials found in config file".into(),
            ));
        }

        // Find the appropriate credential entry
        let (entry_name, entry) = if let Some(requested_name) = name {
            // Look for specific named entry
            config
                .iter()
                .find(|(k, _)| k.as_str() == requested_name)
                .ok_or_else(|| {
                    AuthError::ConfigParseError(format!(
                        "Credential '{}' not found",
                        requested_name
                    ))
                })?
        } else {
            // Look for default entry, or use first one
            config
                .iter()
                .find(|(k, _)| k.starts_with("default"))
                .or_else(|| config.iter().next())
                .ok_or_else(|| AuthError::ConfigParseError("No credentials found".into()))?
        };

        // Determine channel from entry
        let channel = Self::parse_channel_from_entry(entry);

        log::info!(
            "Loaded IBM Quantum credentials from {} [{}]",
            config_path.display(),
            entry_name
        );

        let token_type = TokenType::detect(&entry.token);

        Ok(Self {
            api_token: entry.token.clone(),
            token_type,
            instance: entry.instance.clone(),
            channel,
            service_crn: None,
            cached_iam_token: Arc::new(RwLock::new(None)),
        })
    }

    /// Parse channel type from config entry
    fn parse_channel_from_entry(entry: &QiskitConfigEntry) -> IbmChannel {
        // Check channel field
        if let Some(ref ch) = entry.channel {
            let ch_lower = ch.to_lowercase();
            if ch_lower.contains("cloud") {
                return IbmChannel::IbmCloud;
            }
        }

        // Check URL field
        if let Some(ref url) = entry.url {
            if url.contains("cloud.ibm.com") && !url.contains("quantum-computing") {
                return IbmChannel::IbmCloud;
            }
        }

        IbmChannel::IbmQuantum
    }

    /// Auto-detect credentials from multiple sources (recommended)
    ///
    /// Priority order:
    /// 1. Environment variables (IBM_QUANTUM_TOKEN, IBMQ_TOKEN, QISKIT_IBM_TOKEN)
    /// 2. Qiskit config file (~/.qiskit/qiskit-ibm.json)
    ///
    /// This mimics Python's QiskitRuntimeService() behavior.
    pub fn auto_load() -> Result<Self, AuthError> {
        // Try environment variables first
        if let Ok(creds) = Self::from_env() {
            log::info!("Loaded IBM Quantum credentials from environment variables");
            return Ok(creds);
        }

        // Try Qiskit config file
        if let Ok(creds) = Self::from_qiskit_config(None) {
            return Ok(creds);
        }

        // Nothing found
        Err(AuthError::MissingToken)
    }

    /// Set instance (hub/group/project)
    pub fn with_instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = Some(instance.into());
        self
    }

    /// Set channel
    pub fn with_channel(mut self, channel: IbmChannel) -> Self {
        self.channel = channel;
        self
    }

    /// Set service CRN (Cloud Resource Name) for IBM Cloud
    pub fn with_crn(mut self, crn: impl Into<String>) -> Self {
        self.service_crn = Some(crn.into());
        self
    }

    /// Get API token
    pub fn token(&self) -> &str {
        &self.api_token
    }

    /// Get instance
    pub fn instance(&self) -> Option<&str> {
        self.instance.as_deref()
    }

    /// Get channel
    pub fn channel(&self) -> IbmChannel {
        self.channel
    }

    /// Get service CRN
    pub fn service_crn(&self) -> Option<&str> {
        self.service_crn.as_deref()
    }

    /// Get token type
    pub fn token_type(&self) -> TokenType {
        self.token_type
    }

    /// Check if this is an API key requiring IAM exchange
    pub fn requires_iam_exchange(&self) -> bool {
        self.token_type == TokenType::ApiKey
    }

    /// Get authorization header value (sync version, for IQP tokens only)
    ///
    /// For API keys, use `auth_header_async()` instead.
    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_token)
    }

    /// Get authorization header value (async version, handles IAM exchange)
    ///
    /// For API keys, this will exchange for an IAM access token.
    /// For IQP tokens, this returns the Bearer token directly.
    pub async fn auth_header_async(&self) -> Result<String, AuthError> {
        match self.token_type {
            TokenType::IqpToken => Ok(format!("Bearer {}", self.api_token)),
            TokenType::ApiKey => {
                let access_token = self.get_or_refresh_iam_token().await?;
                Ok(format!("Bearer {}", access_token))
            }
        }
    }

    /// Get or refresh IAM access token
    async fn get_or_refresh_iam_token(&self) -> Result<String, AuthError> {
        // Check cached token first
        {
            let cache = self.cached_iam_token.read().await;
            if let Some(cached) = cache.as_ref() {
                // Return cached token if not expired (with 60s buffer)
                if cached.expires_at > Instant::now() + Duration::from_secs(60) {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Exchange API key for IAM access token
        let access_token = self.exchange_api_key_for_iam_token().await?;

        Ok(access_token)
    }

    /// Exchange API key for IAM access token
    async fn exchange_api_key_for_iam_token(&self) -> Result<String, AuthError> {
        let client = reqwest::Client::new();

        // Strip "ApiKey-" prefix if present
        let api_key = self
            .api_token
            .strip_prefix("ApiKey-")
            .or_else(|| self.api_token.strip_prefix("apikey-"))
            .unwrap_or(&self.api_token);

        let params = [
            ("grant_type", "urn:ibm:params:oauth:grant-type:apikey"),
            ("apikey", api_key),
        ];

        let response = client
            .post(IAM_TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AuthError::IamTokenExchangeFailed(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let token_response: IamTokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::IamTokenExchangeFailed(e.to_string()))?;

        // Cache the token
        let expires_at = Instant::now() + Duration::from_secs(token_response.expires_in);
        {
            let mut cache = self.cached_iam_token.write().await;
            *cache = Some(CachedToken {
                access_token: token_response.access_token.clone(),
                expires_at,
            });
        }

        log::info!(
            "IAM token obtained, expires in {}s",
            token_response.expires_in
        );

        Ok(token_response.access_token)
    }

    /// Validate token format (basic check)
    pub fn validate(&self) -> Result<(), AuthError> {
        if self.api_token.is_empty() {
            return Err(AuthError::MissingToken);
        }

        // API keys have format ApiKey-xxx (40+ chars)
        // IQP tokens are typically 64+ characters
        let min_len = match self.token_type {
            TokenType::ApiKey => 32,
            TokenType::IqpToken => 32,
        };

        if self.api_token.len() < min_len {
            return Err(AuthError::InvalidTokenFormat);
        }

        Ok(())
    }
}

/// Token information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// User ID
    #[serde(rename = "userId")]
    pub user_id: Option<String>,

    /// Token expiry
    #[serde(rename = "exp")]
    pub expires_at: Option<i64>,

    /// Token issued at
    #[serde(rename = "iat")]
    pub issued_at: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_new() {
        let creds = IbmCredentials::new("test_token_12345678901234567890123456789012");
        assert_eq!(creds.token(), "test_token_12345678901234567890123456789012");
        assert!(creds.instance().is_none());
        assert_eq!(creds.channel(), IbmChannel::IbmQuantum);
    }

    #[test]
    fn test_credentials_with_instance() {
        let creds = IbmCredentials::new("test_token_12345678901234567890123456789012")
            .with_instance("ibm-q/open/main");

        assert_eq!(creds.instance(), Some("ibm-q/open/main"));
    }

    #[test]
    fn test_channel_urls() {
        // Both channels now use the unified quantum.cloud.ibm.com endpoint (2025 API)
        assert!(IbmChannel::IbmQuantum
            .base_url()
            .contains("quantum.cloud.ibm.com"));
        assert!(IbmChannel::IbmCloud
            .base_url()
            .contains("quantum.cloud.ibm.com"));
    }

    #[test]
    fn test_token_type_detection() {
        // API key format
        assert_eq!(TokenType::detect("ApiKey-abc123def456"), TokenType::ApiKey);
        assert_eq!(TokenType::detect("apikey-abc123def456"), TokenType::ApiKey);

        // IQP token format
        assert_eq!(
            TokenType::detect("EXzoQd612YxXnPdUcup2cUBWIDXJmIsEOUw0acu83FRB"),
            TokenType::IqpToken
        );
        assert_eq!(TokenType::detect("some_regular_token"), TokenType::IqpToken);
    }

    #[test]
    fn test_requires_iam_exchange() {
        let api_key_creds = IbmCredentials::new("ApiKey-test12345678901234567890");
        assert!(api_key_creds.requires_iam_exchange());

        let iqp_creds = IbmCredentials::new("regular_iqp_token_1234567890123456789012");
        assert!(!iqp_creds.requires_iam_exchange());
    }

    #[test]
    fn test_validate_token() {
        let valid = IbmCredentials::new("a".repeat(64));
        assert!(valid.validate().is_ok());

        let short = IbmCredentials::new("short");
        assert!(matches!(
            short.validate(),
            Err(AuthError::InvalidTokenFormat)
        ));

        let empty = IbmCredentials::new("");
        assert!(matches!(empty.validate(), Err(AuthError::MissingToken)));
    }

    #[test]
    fn test_auth_header() {
        let creds = IbmCredentials::new("my_token");
        assert_eq!(creds.auth_header(), "Bearer my_token");
    }

    #[test]
    fn test_parse_qiskit_config_json() {
        // Test parsing of qiskit-ibm.json format
        let json = r#"{
            "default-ibm-quantum-platform": {
                "channel": "ibm_quantum_platform",
                "token": "test_token_1234567890abcdef",
                "url": "https://cloud.ibm.com"
            },
            "my-premium": {
                "channel": "ibm_cloud",
                "token": "premium_token_xyz",
                "instance": "crn:v1:bluemix:..."
            }
        }"#;

        let config: HashMap<String, QiskitConfigEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(config.len(), 2);
        assert!(config.contains_key("default-ibm-quantum-platform"));
        assert!(config.contains_key("my-premium"));

        let default_entry = &config["default-ibm-quantum-platform"];
        assert_eq!(default_entry.token, "test_token_1234567890abcdef");
    }

    #[test]
    fn test_parse_channel_from_entry() {
        // Test IBM Quantum channel detection
        let quantum_entry = QiskitConfigEntry {
            token: "test".into(),
            channel: Some("ibm_quantum_platform".into()),
            url: None,
            instance: None,
            plans_preference: None,
        };
        assert_eq!(
            IbmCredentials::parse_channel_from_entry(&quantum_entry),
            IbmChannel::IbmQuantum
        );

        // Test IBM Cloud channel detection
        let cloud_entry = QiskitConfigEntry {
            token: "test".into(),
            channel: Some("ibm_cloud".into()),
            url: None,
            instance: None,
            plans_preference: None,
        };
        assert_eq!(
            IbmCredentials::parse_channel_from_entry(&cloud_entry),
            IbmChannel::IbmCloud
        );
    }

    #[test]
    fn test_qiskit_config_path() {
        // Should return a path ending with .qiskit/qiskit-ibm.json
        if let Some(path) = IbmCredentials::qiskit_config_path() {
            assert!(
                path.ends_with(".qiskit/qiskit-ibm.json")
                    || path.ends_with(".qiskit\\qiskit-ibm.json")
            );
        }
    }

    #[test]
    fn test_auto_load_from_qiskit_config() {
        // This test will pass if ~/.qiskit/qiskit-ibm.json exists
        // Otherwise it will try env vars, and if neither exists, it will fail gracefully
        let result = IbmCredentials::auto_load();
        // We just verify it doesn't panic - actual success depends on system config
        match result {
            Ok(creds) => {
                assert!(!creds.token().is_empty());
            }
            Err(AuthError::MissingToken) => {
                // Expected if no credentials are configured
            }
            Err(e) => {
                // Other errors are also acceptable in test environment
                println!("auto_load returned: {:?}", e);
            }
        }
    }
}
