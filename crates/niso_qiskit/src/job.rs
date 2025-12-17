//! IBM Quantum job management
//!
//! Gantree: L10_Qiskit → Job

use crate::client::{ClientError, IbmClient};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

/// Job errors
#[derive(Debug, Error)]
pub enum JobError {
    /// Client error
    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    /// Job submission failed
    #[error("Job submission failed: {0}")]
    SubmissionFailed(String),

    /// Job execution failed
    #[error("Job execution failed: {0}")]
    ExecutionFailed(String),

    /// Job cancelled
    #[error("Job was cancelled")]
    Cancelled,

    /// Job timeout
    #[error("Job timed out after {0} seconds")]
    Timeout(u64),

    /// Invalid job state
    #[error("Invalid job state: {0}")]
    InvalidState(String),

    /// Result not available
    #[error("Job results not yet available")]
    ResultsNotReady,
}

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    /// Job is queued
    #[serde(
        alias = "queued",
        alias = "Queued",
        alias = "PENDING",
        alias = "pending"
    )]
    Queued,

    /// Job is validating
    #[serde(alias = "validating", alias = "Validating")]
    Validating,

    /// Job is running
    #[serde(alias = "running", alias = "Running")]
    Running,

    /// Job completed successfully
    #[serde(
        alias = "completed",
        alias = "Completed",
        alias = "DONE",
        alias = "done"
    )]
    Completed,

    /// Job failed
    #[serde(alias = "failed", alias = "Failed", alias = "ERROR", alias = "error")]
    Failed,

    /// Job was cancelled
    #[serde(
        alias = "cancelled",
        alias = "Cancelled",
        alias = "CANCELED",
        alias = "canceled"
    )]
    Cancelled,

    /// Unknown status
    #[serde(other)]
    Unknown,
}

impl JobStatus {
    /// Check if job is in terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Check if job is still running
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Queued | Self::Validating | Self::Running)
    }

    /// Check if job completed successfully
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// Job submission request
#[derive(Debug, Clone, Serialize)]
pub struct JobSubmission {
    /// Program ID (e.g., "sampler", "estimator")
    pub program_id: String,

    /// Backend name
    pub backend: String,

    /// Input parameters
    pub params: JobParams,

    /// Job tags
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Job parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobParams {
    /// Circuits (as OpenQASM strings)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub circuits: Vec<String>,

    /// Number of shots
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shots: Option<u64>,

    /// Seed for simulator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_simulator: Option<u64>,

    /// Skip transpilation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_transpilation: Option<bool>,

    /// Optimization level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimization_level: Option<u8>,

    /// Additional options
    #[serde(flatten)]
    pub options: HashMap<String, serde_json::Value>,
}

impl Default for JobParams {
    fn default() -> Self {
        Self {
            circuits: Vec::new(),
            shots: Some(4096),
            seed_simulator: None,
            skip_transpilation: None,
            optimization_level: Some(1),
            options: HashMap::new(),
        }
    }
}

impl JobParams {
    /// Create new params with circuits
    pub fn new(circuits: Vec<String>) -> Self {
        Self {
            circuits,
            ..Default::default()
        }
    }

    /// Set number of shots
    pub fn with_shots(mut self, shots: u64) -> Self {
        self.shots = Some(shots);
        self
    }

    /// Set seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed_simulator = Some(seed);
        self
    }

    /// Set optimization level
    pub fn with_optimization_level(mut self, level: u8) -> Self {
        self.optimization_level = Some(level);
        self
    }

    /// Skip transpilation
    pub fn skip_transpilation(mut self) -> Self {
        self.skip_transpilation = Some(true);
        self
    }
}

/// Job submission response (2025 API)
#[derive(Debug, Clone, Deserialize)]
pub struct JobResponse {
    /// Job ID
    pub id: String,

    /// Backend
    pub backend: Option<String>,

    /// Status (optional in 2025 API - may not be present on submit)
    #[serde(default)]
    pub status: Option<JobStatus>,

    /// State (2025 API alternative to status)
    pub state: Option<JobState>,

    /// Creation time
    pub created: Option<DateTime<Utc>>,

    /// Error message
    pub error: Option<JobErrorInfo>,
}

/// Job state (2025 API format)
#[derive(Debug, Clone, Deserialize)]
pub struct JobState {
    /// Status string
    pub status: Option<String>,
    /// Reason
    pub reason: Option<String>,
}

impl JobResponse {
    /// Get effective job status
    pub fn effective_status(&self) -> JobStatus {
        // Try direct status first
        if let Some(status) = self.status {
            return status;
        }

        // Try state.status (2025 API format)
        if let Some(ref state) = self.state {
            if let Some(ref status_str) = state.status {
                return match status_str.to_uppercase().as_str() {
                    "QUEUED" => JobStatus::Queued,
                    "VALIDATING" => JobStatus::Validating,
                    "RUNNING" => JobStatus::Running,
                    "COMPLETED" | "DONE" => JobStatus::Completed,
                    "FAILED" | "ERROR" => JobStatus::Failed,
                    "CANCELLED" | "CANCELED" => JobStatus::Cancelled,
                    _ => JobStatus::Queued, // Default to queued if unknown
                };
            }
        }

        // Default: just submitted, assume queued
        JobStatus::Queued
    }
}

/// Job error information
#[derive(Debug, Clone, Deserialize)]
pub struct JobErrorInfo {
    /// Error message
    pub message: Option<String>,

    /// Error code
    pub code: Option<i32>,
}

/// Job result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Job ID
    pub job_id: Option<String>,

    /// Results per circuit
    pub results: Vec<CircuitResult>,

    /// Metadata
    pub metadata: Option<JobMetadata>,
}

/// Single circuit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitResult {
    /// Measurement counts
    pub counts: Option<HashMap<String, u64>>,

    /// Memory (individual shot results)
    pub memory: Option<Vec<String>>,

    /// Success flag
    #[serde(default)]
    pub success: bool,

    /// Number of shots
    pub shots: Option<u64>,

    /// Execution time
    pub time_taken: Option<f64>,
}

/// Job metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    /// Total execution time
    pub time_taken: Option<f64>,

    /// Number of circuits
    pub num_circuits: Option<usize>,
}

/// IBM Quantum Job handle
pub struct IbmJob {
    /// Job ID
    id: String,

    /// Client
    client: IbmClient,

    /// Cached status
    status: JobStatus,

    /// Backend name
    backend: String,
}

impl IbmJob {
    /// Create job handle from response
    pub(crate) fn new(response: JobResponse, client: IbmClient) -> Self {
        let status = response.effective_status();
        Self {
            id: response.id,
            status,
            backend: response.backend.unwrap_or_default(),
            client,
        }
    }

    /// Get job ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get backend name
    pub fn backend(&self) -> &str {
        &self.backend
    }

    /// Get current status
    pub fn status(&self) -> JobStatus {
        self.status
    }

    /// Refresh job status
    pub async fn refresh(&mut self) -> Result<JobStatus, JobError> {
        let path = format!("/jobs/{}", self.id);
        let response: JobResponse = self.client.get(&path).await?;
        self.status = response.effective_status();

        if let Some(error) = response.error {
            if self.status == JobStatus::Failed {
                return Err(JobError::ExecutionFailed(
                    error.message.unwrap_or_else(|| "Unknown error".to_string()),
                ));
            }
        } else if let Some(state) = response.state {
            if self.status == JobStatus::Failed {
                if let Some(reason) = state.reason {
                    return Err(JobError::ExecutionFailed(reason));
                }
            }
        }

        Ok(self.status)
    }

    /// Wait for job completion
    pub async fn wait(&mut self, timeout: Duration) -> Result<JobStatus, JobError> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_secs(5);

        loop {
            let status = self.refresh().await?;

            if status.is_terminal() {
                return match status {
                    JobStatus::Completed => Ok(status),
                    JobStatus::Failed => Err(JobError::ExecutionFailed("Job failed".to_string())),
                    JobStatus::Cancelled => Err(JobError::Cancelled),
                    _ => Ok(status),
                };
            }

            if start.elapsed() > timeout {
                return Err(JobError::Timeout(timeout.as_secs()));
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Get job results
    pub async fn result(&self) -> Result<JobResult, JobError> {
        if !self.status.is_terminal() {
            return Err(JobError::ResultsNotReady);
        }

        let path = format!("/jobs/{}/results", self.id);
        let result: JobResult = self.client.get(&path).await?;
        Ok(result)
    }

    /// Cancel job
    pub async fn cancel(&mut self) -> Result<(), JobError> {
        let path = format!("/jobs/{}", self.id);
        self.client.delete(&path).await?;
        self.status = JobStatus::Cancelled;
        Ok(())
    }
}

/// Job manager for submitting and tracking jobs
pub struct JobManager {
    client: IbmClient,
}

impl JobManager {
    /// Create new job manager
    pub fn new(client: IbmClient) -> Self {
        Self { client }
    }

    /// Submit a job
    pub async fn submit(&self, submission: JobSubmission) -> Result<IbmJob, JobError> {
        let response: JobResponse = self.client.post("/jobs", &submission).await?;

        if response.effective_status() == JobStatus::Failed {
            return Err(JobError::SubmissionFailed(
                response
                    .error
                    .and_then(|e| e.message)
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        Ok(IbmJob::new(response, self.client.clone()))
    }

    /// Submit sampler job
    pub async fn submit_sampler(
        &self,
        backend: &str,
        circuits: Vec<String>,
        shots: u64,
    ) -> Result<IbmJob, JobError> {
        let submission = JobSubmission {
            program_id: "sampler".to_string(),
            backend: backend.to_string(),
            params: JobParams::new(circuits).with_shots(shots),
            tags: vec!["niso".to_string()],
        };

        self.submit(submission).await
    }

    /// Get existing job by ID
    pub async fn get_job(&self, job_id: &str) -> Result<IbmJob, JobError> {
        let path = format!("/jobs/{}", job_id);
        let response: JobResponse = self.client.get(&path).await?;
        Ok(IbmJob::new(response, self.client.clone()))
    }

    /// List recent jobs
    pub async fn list_jobs(&self, limit: usize) -> Result<Vec<JobResponse>, JobError> {
        let path = format!("/jobs?limit={}", limit);
        let response: JobsListResponse = self.client.get(&path).await?;
        Ok(response.jobs)
    }
}

/// Jobs list response
#[derive(Debug, Deserialize)]
struct JobsListResponse {
    jobs: Vec<JobResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_terminal() {
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(!JobStatus::Queued.is_terminal());
    }

    #[test]
    fn test_job_status_running() {
        assert!(JobStatus::Running.is_running());
        assert!(JobStatus::Queued.is_running());
        assert!(!JobStatus::Completed.is_running());
    }

    #[test]
    fn test_job_params_builder() {
        let params = JobParams::new(vec!["OPENQASM 3;".to_string()])
            .with_shots(8192)
            .with_seed(42)
            .with_optimization_level(2);

        assert_eq!(params.shots, Some(8192));
        assert_eq!(params.seed_simulator, Some(42));
        assert_eq!(params.optimization_level, Some(2));
    }

    #[test]
    fn test_job_response_deserialize() {
        // Legacy format with status
        let json = r#"{
            "id": "job_12345",
            "backend": "ibm_brisbane",
            "status": "QUEUED"
        }"#;

        let response: JobResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "job_12345");
        assert_eq!(response.effective_status(), JobStatus::Queued);

        // 2025 API format (no status on submit)
        let json_2025 = r#"{
            "id": "d4lnn12v0j9c73e5h490",
            "backend": "ibm_fez"
        }"#;

        let response_2025: JobResponse = serde_json::from_str(json_2025).unwrap();
        assert_eq!(response_2025.id, "d4lnn12v0j9c73e5h490");
        assert_eq!(response_2025.effective_status(), JobStatus::Queued); // Default
    }

    #[test]
    fn test_circuit_result_deserialize() {
        let json = r#"{
            "counts": {"00": 512, "11": 512},
            "success": true,
            "shots": 1024
        }"#;

        let result: CircuitResult = serde_json::from_str(json).unwrap();
        assert!(result.success);
        assert_eq!(result.shots, Some(1024));

        let counts = result.counts.unwrap();
        assert_eq!(counts.get("00"), Some(&512));
        assert_eq!(counts.get("11"), Some(&512));
    }
}
