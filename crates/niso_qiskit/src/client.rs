//! IBM Quantum REST API client
//!
//! Gantree: L10_Qiskit → Client

use crate::auth::{AuthError, IbmCredentials, TokenType};
use reqwest::header::{
    HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

/// Client errors
#[derive(Debug, Error)]
pub enum ClientError {
    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// API error response
    #[error("API error ({code}): {message}")]
    ApiError {
        /// HTTP status code
        code: u16,
        /// Error message
        message: String,
    },

    /// JSON parsing error
    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Backend not found
    #[error("Backend not found: {0}")]
    BackendNotFound(String),

    /// Job not found
    #[error("Job not found: {0}")]
    JobNotFound(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded, retry after {retry_after}s")]
    RateLimited {
        /// Seconds to wait before retry
        retry_after: u64,
    },

    /// Service unavailable
    #[error("Service temporarily unavailable")]
    ServiceUnavailable,
}

/// IBM Quantum API client
#[derive(Clone)]
pub struct IbmClient {
    /// HTTP client
    client: reqwest::Client,

    /// Credentials
    credentials: IbmCredentials,

    /// Base URL
    base_url: String,
}

impl IbmClient {
    /// Create new client with credentials (sync, for IQP tokens)
    ///
    /// For API keys, use `new_async()` instead.
    pub fn new(credentials: IbmCredentials) -> Result<Self, ClientError> {
        credentials.validate()?;

        let base_url = credentials.channel().runtime_url();

        let mut headers = HeaderMap::new();
        // Note: For API keys, the auth header will be set per-request
        if credentials.token_type() == TokenType::IqpToken {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&credentials.auth_header())
                    .map_err(|_| AuthError::InvalidTokenFormat)?,
            );
        }
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // User-Agent to avoid Cloudflare blocking
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("niso-qiskit/0.1.0 (Rust; NISO Quantum Optimizer)"),
        );

        // Required for 2025 API
        headers.insert(
            HeaderName::from_static("ibm-api-version"),
            HeaderValue::from_static("2025-01-01"),
        );

        // Accept header
        headers.insert(
            HeaderName::from_static("accept"),
            HeaderValue::from_static("application/json"),
        );

        // Service CRN header (required for IBM Cloud)
        if let Some(crn) = credentials.service_crn() {
            headers.insert(
                HeaderName::from_static("service-crn"),
                HeaderValue::from_str(crn).map_err(|_| AuthError::InvalidTokenFormat)?,
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        Ok(Self {
            client,
            credentials,
            base_url,
        })
    }

    /// Create new client with credentials (async, handles IAM exchange for API keys)
    pub async fn new_async(credentials: IbmCredentials) -> Result<Self, ClientError> {
        credentials.validate()?;

        // Pre-cache the IAM token (but don't set it in default headers)
        // This ensures the token is ready and validates the API key
        let _ = credentials.auth_header_async().await?;

        let base_url = credentials.channel().runtime_url();

        let mut headers = HeaderMap::new();
        // Note: For API keys, we DON'T set AUTHORIZATION in default headers
        // It will be set per-request via get_auth_header() to handle token refresh
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // User-Agent to avoid Cloudflare blocking
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("niso-qiskit/0.1.0 (Rust; NISO Quantum Optimizer)"),
        );

        // Required for 2025 API
        headers.insert(
            HeaderName::from_static("ibm-api-version"),
            HeaderValue::from_static("2025-01-01"),
        );

        // Accept header
        headers.insert(
            HeaderName::from_static("accept"),
            HeaderValue::from_static("application/json"),
        );

        // Service CRN header (required for IBM Cloud)
        if let Some(crn) = credentials.service_crn() {
            headers.insert(
                HeaderName::from_static("service-crn"),
                HeaderValue::from_str(crn).map_err(|_| AuthError::InvalidTokenFormat)?,
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        Ok(Self {
            client,
            credentials,
            base_url,
        })
    }

    /// Create from environment
    pub fn from_env() -> Result<Self, ClientError> {
        let credentials = IbmCredentials::from_env()?;
        Self::new(credentials)
    }

    // ========================================================================
    // Low-level HTTP methods
    // ========================================================================

    /// Get fresh authorization header (handles token refresh for API keys)
    async fn get_auth_header(&self) -> Result<String, ClientError> {
        if self.credentials.requires_iam_exchange() {
            // For API keys, always get fresh/cached token (handles expiry)
            Ok(self.credentials.auth_header_async().await?)
        } else {
            // For IQP tokens, use directly
            Ok(self.credentials.auth_header())
        }
    }

    /// GET request
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        let url = format!("{}{}", self.base_url, path);

        // Get fresh auth header for API keys
        let mut request = self.client.get(&url);
        if self.credentials.requires_iam_exchange() {
            let auth = self.get_auth_header().await?;
            request = request.header(AUTHORIZATION, auth);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// POST request
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ClientError> {
        let url = format!("{}{}", self.base_url, path);

        // Get fresh auth header for API keys
        let mut request = self.client.post(&url).json(body);
        if self.credentials.requires_iam_exchange() {
            let auth = self.get_auth_header().await?;
            request = request.header(AUTHORIZATION, auth);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// DELETE request
    pub async fn delete(&self, path: &str) -> Result<(), ClientError> {
        let url = format!("{}{}", self.base_url, path);

        // Get fresh auth header for API keys
        let mut request = self.client.delete(&url);
        if self.credentials.requires_iam_exchange() {
            let auth = self.get_auth_header().await?;
            request = request.header(AUTHORIZATION, auth);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            Err(ClientError::ApiError {
                code: status,
                message: text,
            })
        }
    }

    /// Handle API response
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, ClientError> {
        let status = response.status();

        if status.is_success() {
            let body = response.text().await?;
            serde_json::from_str(&body).map_err(ClientError::from)
        } else {
            match status.as_u16() {
                401 => Err(ClientError::Auth(AuthError::AuthFailed(
                    "Invalid or expired token".to_string(),
                ))),
                404 => {
                    let text = response.text().await.unwrap_or_default();
                    Err(ClientError::ApiError {
                        code: 404,
                        message: text,
                    })
                }
                429 => {
                    let retry_after = response
                        .headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(60);
                    Err(ClientError::RateLimited { retry_after })
                }
                503 => Err(ClientError::ServiceUnavailable),
                _ => {
                    let text = response.text().await.unwrap_or_default();
                    Err(ClientError::ApiError {
                        code: status.as_u16(),
                        message: text,
                    })
                }
            }
        }
    }

    /// GET request with retry
    pub async fn get_with_retry<T: DeserializeOwned>(
        &self,
        path: &str,
        max_retries: usize,
    ) -> Result<T, ClientError> {
        let mut retries = 0;
        loop {
            match self.get(path).await {
                Ok(result) => return Ok(result),
                Err(ClientError::RateLimited { retry_after }) => {
                    if retries >= max_retries {
                        return Err(ClientError::RateLimited { retry_after });
                    }
                    retries += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(retry_after)).await;
                }
                Err(ClientError::ServiceUnavailable) => {
                    if retries >= max_retries {
                        return Err(ClientError::ServiceUnavailable);
                    }
                    retries += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// POST request with retry
    pub async fn post_with_retry<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        max_retries: usize,
    ) -> Result<T, ClientError> {
        let mut retries = 0;
        loop {
            match self.post(path, body).await {
                Ok(result) => return Ok(result),
                Err(ClientError::RateLimited { retry_after }) => {
                    if retries >= max_retries {
                        return Err(ClientError::RateLimited { retry_after });
                    }
                    retries += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(retry_after)).await;
                }
                Err(ClientError::ServiceUnavailable) => {
                    if retries >= max_retries {
                        return Err(ClientError::ServiceUnavailable);
                    }
                    retries += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    // ========================================================================
    // Backend APIs
    // ========================================================================

    /// List available backends
    pub async fn list_backends(&self) -> Result<Vec<BackendInfo>, ClientError> {
        // Get raw response to see the actual structure
        let url = format!("{}/backends", self.base_url);

        // Get fresh auth header for API keys
        let mut request = self.client.get(&url);
        if self.credentials.requires_iam_exchange() {
            let auth = self.get_auth_header().await?;
            request = request.header(AUTHORIZATION, auth);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ClientError::ApiError {
                code: status,
                message: text,
            });
        }

        let body = response.text().await?;

        // Try parsing as DevicesResponse first (2025 API: {"devices": [...]})
        if let Ok(response) = serde_json::from_str::<DevicesResponse>(&body) {
            return Ok(response
                .devices
                .into_iter()
                .map(|d| d.to_backend_info())
                .collect());
        }

        // Try parsing as BackendsResponse (legacy: {"backends": [...]})
        if let Ok(response) = serde_json::from_str::<BackendsResponse>(&body) {
            return Ok(response.backends);
        }

        // Try parsing as direct array ([...])
        if let Ok(backends) = serde_json::from_str::<Vec<BackendInfo>>(&body) {
            return Ok(backends);
        }

        // Log the actual response for debugging
        eprintln!(
            "Unexpected backends response format. First 500 chars: {}",
            &body[..body.len().min(500)]
        );

        // Force a parse error for unknown format
        Err(ClientError::ApiError {
            code: 500,
            message: "Unknown response format from backends API".to_string(),
        })
    }

    /// Get backend details
    pub async fn get_backend(&self, name: &str) -> Result<BackendInfo, ClientError> {
        let path = format!("/backends/{}", name);
        self.get(&path).await
    }

    /// Get backend status
    pub async fn get_backend_status(&self, name: &str) -> Result<BackendStatus, ClientError> {
        let path = format!("/backends/{}/status", name);
        self.get(&path).await
    }

    /// Get backend configuration
    pub async fn get_backend_config(&self, name: &str) -> Result<BackendConfig, ClientError> {
        let path = format!("/backends/{}/configuration", name);
        self.get(&path).await
    }

    /// Get backend properties (calibration data)
    pub async fn get_backend_properties(
        &self,
        name: &str,
    ) -> Result<BackendProperties, ClientError> {
        let path = format!("/backends/{}/properties", name);
        self.get(&path).await
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get credentials
    pub fn credentials(&self) -> &IbmCredentials {
        &self.credentials
    }

    /// Get base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

// ============================================================================
// Response Types
// ============================================================================

/// Backends list response (legacy format)
#[derive(Debug, Clone, Deserialize)]
pub struct BackendsResponse {
    /// List of backends
    pub backends: Vec<BackendInfo>,
}

/// Devices list response (2025 API format)
#[derive(Debug, Clone, Deserialize)]
pub struct DevicesResponse {
    /// List of devices
    pub devices: Vec<DeviceInfo>,
}

/// Device information (2025 API format)
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceInfo {
    /// Backend name
    pub name: String,

    /// Number of qubits
    #[serde(default)]
    pub qubits: usize,

    /// Processor type
    pub processor_type: Option<ProcessorType>,

    /// Queue length
    #[serde(default)]
    pub queue_length: u64,

    /// Status
    pub status: Option<DeviceStatus>,

    /// CLOPS (Circuit Layer Operations Per Second)
    pub clops: Option<ClopsInfo>,

    /// Performance metrics
    pub performance_metrics: Option<PerformanceMetrics>,
}

/// Device status (2025 API)
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceStatus {
    /// Status name (e.g., "online", "offline")
    pub name: Option<String>,
    /// Status reason
    pub reason: Option<String>,
}

/// CLOPS info
#[derive(Debug, Clone, Deserialize)]
pub struct ClopsInfo {
    /// Type (hardware/simulator)
    #[serde(rename = "type")]
    pub clops_type: Option<String>,
    /// Value
    pub value: Option<u64>,
}

/// Performance metrics
#[derive(Debug, Clone, Deserialize)]
pub struct PerformanceMetrics {
    /// Two-qubit error median
    pub two_q_error_median: Option<MetricValue>,
    /// Two-qubit error best
    pub two_q_error_best: Option<TwoQErrorBest>,
    /// Readout error median
    pub readout_error_median: Option<MetricValue>,
}

/// Metric value
#[derive(Debug, Clone, Deserialize)]
pub struct MetricValue {
    /// Unit
    pub unit: Option<String>,
    /// Value
    pub value: Option<f64>,
}

/// Two-qubit error best
#[derive(Debug, Clone, Deserialize)]
pub struct TwoQErrorBest {
    /// Gate type
    pub gate: Option<String>,
    /// Qubits
    pub qubits: Option<Vec<usize>>,
    /// Value
    pub value: Option<f64>,
}

impl DeviceInfo {
    /// Convert to BackendInfo for compatibility
    pub fn to_backend_info(&self) -> BackendInfo {
        let is_online = self
            .status
            .as_ref()
            .and_then(|s| s.name.as_ref())
            .map(|n| n == "online")
            .unwrap_or(false);

        BackendInfo {
            name: self.name.clone(),
            num_qubits: Some(self.qubits),
            simulator: self
                .clops
                .as_ref()
                .and_then(|c| c.clops_type.as_ref())
                .map(|t| t == "simulator")
                .unwrap_or(false),
            version: self
                .processor_type
                .as_ref()
                .and_then(|p| p.revision.clone()),
            description: None,
            processor_type: self.processor_type.clone(),
            operational: is_online,
            queue_length: Some(self.queue_length),
        }
    }
}

/// Backend information (unified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendInfo {
    /// Backend name
    pub name: String,

    /// Number of qubits
    #[serde(rename = "n_qubits", alias = "num_qubits", alias = "qubits")]
    pub num_qubits: Option<usize>,

    /// Whether backend is simulator
    #[serde(default)]
    pub simulator: bool,

    /// Backend version
    pub version: Option<String>,

    /// Backend description
    pub description: Option<String>,

    /// Processor type
    pub processor_type: Option<ProcessorType>,

    /// Whether backend is operational
    #[serde(default)]
    pub operational: bool,

    /// Queue length
    #[serde(default)]
    pub queue_length: Option<u64>,
}

/// Processor type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorType {
    /// Family (e.g., "Falcon", "Heron")
    pub family: Option<String>,

    /// Revision
    pub revision: Option<String>,
}

/// Backend status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStatus {
    /// Backend name
    pub backend_name: Option<String>,

    /// Operational status
    #[serde(default)]
    pub operational: bool,

    /// Pending jobs
    pub pending_jobs: Option<u64>,

    /// Status message
    pub status_msg: Option<String>,
}

/// Backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Backend name
    pub backend_name: Option<String>,

    /// Number of qubits
    pub n_qubits: Option<usize>,

    /// Basis gates
    pub basis_gates: Option<Vec<String>>,

    /// Coupling map
    pub coupling_map: Option<Vec<Vec<usize>>>,

    /// Max shots
    pub max_shots: Option<u64>,

    /// Max experiments
    pub max_experiments: Option<usize>,

    /// Supported features
    pub supported_features: Option<Vec<String>>,
}

/// Backend properties (calibration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendProperties {
    /// Last update time
    pub last_update_date: Option<String>,

    /// Qubit properties
    pub qubits: Option<Vec<Vec<QubitProperty>>>,

    /// Gate properties
    pub gates: Option<Vec<GateProperty>>,
}

/// Single qubit property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QubitProperty {
    /// Property name
    pub name: String,

    /// Property value
    pub value: f64,

    /// Unit
    pub unit: Option<String>,

    /// Date
    pub date: Option<String>,
}

/// Gate property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateProperty {
    /// Gate name
    pub gate: String,

    /// Qubits
    pub qubits: Vec<usize>,

    /// Parameters
    pub parameters: Vec<GateParameter>,
}

/// Gate parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateParameter {
    /// Parameter name
    pub name: String,

    /// Parameter value
    pub value: f64,

    /// Unit
    pub unit: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_info_deserialize() {
        let json = r#"{
            "name": "ibm_brisbane",
            "n_qubits": 127,
            "simulator": false,
            "version": "1.0.0"
        }"#;

        let info: BackendInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "ibm_brisbane");
        assert_eq!(info.num_qubits, Some(127));
        assert!(!info.simulator);
    }

    #[test]
    fn test_backend_status_deserialize() {
        let json = r#"{
            "backend_name": "ibm_brisbane",
            "operational": true,
            "pending_jobs": 42,
            "status_msg": "active"
        }"#;

        let status: BackendStatus = serde_json::from_str(json).unwrap();
        assert!(status.operational);
        assert_eq!(status.pending_jobs, Some(42));
    }

    #[test]
    fn test_backend_properties_deserialize() {
        let json = r#"{
            "last_update_date": "2025-01-01T00:00:00Z",
            "qubits": [[
                {"name": "T1", "value": 100.0, "unit": "us"},
                {"name": "T2", "value": 80.0, "unit": "us"}
            ]],
            "gates": [{
                "gate": "cx",
                "qubits": [0, 1],
                "parameters": [{"name": "gate_error", "value": 0.01}]
            }]
        }"#;

        let props: BackendProperties = serde_json::from_str(json).unwrap();
        assert!(props.qubits.is_some());
        assert!(props.gates.is_some());
    }
}
