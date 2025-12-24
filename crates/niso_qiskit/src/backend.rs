//! IBM Quantum backend implementation
//!
//! Gantree: L10_Qiskit → IbmBackend

use crate::auth::{AuthError, IbmCredentials};
use crate::client::{BackendConfig, BackendProperties, ClientError, IbmClient};
use crate::job::{JobError, JobManager, JobParams, JobSubmission};
use crate::transpiler::{Transpiler, TranspilerConfig};
use niso_backend::{Backend, ExecutionMetadata, ExecutionResult};
use niso_calibration::CalibrationInfo;
use niso_core::{Circuit, NisoError, NisoResult};
use std::time::Duration;
use thiserror::Error;
use tokio::runtime::Runtime;

/// IBM Backend errors
#[derive(Debug, Error)]
pub enum IbmBackendError {
    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    /// Client error
    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    /// Job error
    #[error("Job error: {0}")]
    Job(#[from] JobError),

    /// Transpilation error
    #[error("Transpilation error: {0}")]
    Transpilation(String),

    /// Backend not available
    #[error("Backend {0} is not available")]
    NotAvailable(String),

    /// Runtime error
    #[error("Async runtime error: {0}")]
    Runtime(String),
}

/// IBM Quantum backend
pub struct IbmBackend {
    /// Backend name
    name: String,

    /// API client
    client: IbmClient,

    /// Job manager
    job_manager: JobManager,

    /// Transpiler
    transpiler: Transpiler,

    /// Cached configuration
    config: Option<BackendConfig>,

    /// Cached properties
    properties: Option<BackendProperties>,

    /// Cached calibration
    calibration: Option<CalibrationInfo>,

    /// Tokio runtime for async operations
    runtime: Runtime,

    /// Job timeout
    timeout: Duration,

    /// Use QASM 3 (vs QASM 2)
    use_qasm3: bool,

    /// Number of qubits
    num_qubits: usize,
}

impl IbmBackend {
    /// Create new IBM backend
    ///
    /// Handles both IQP tokens and API keys (with automatic IAM exchange).
    pub fn new(
        name: impl Into<String>,
        credentials: IbmCredentials,
    ) -> Result<Self, IbmBackendError> {
        let name = name.into();

        let runtime = Runtime::new().map_err(|e| IbmBackendError::Runtime(e.to_string()))?;

        // Create client (async for API keys, sync for IQP tokens)
        let client = if credentials.requires_iam_exchange() {
            runtime.block_on(async { IbmClient::new_async(credentials).await })?
        } else {
            IbmClient::new(credentials)?
        };

        let job_manager = JobManager::new(client.clone());

        // Fetch backend config
        let config: Option<BackendConfig> =
            runtime.block_on(async { client.get_backend_config(&name).await.ok() });

        let num_qubits = config.as_ref().and_then(|c| c.n_qubits).unwrap_or(127);

        // Configure transpiler
        let transpiler_config = if let Some(ref cfg) = config {
            let coupling = cfg.coupling_map.as_ref().map(|cm| {
                cm.iter()
                    .filter_map(|pair| {
                        if pair.len() >= 2 {
                            Some((pair[0], pair[1]))
                        } else {
                            None
                        }
                    })
                    .collect()
            });

            TranspilerConfig {
                num_qubits,
                coupling_map: coupling,
                basis_gates: cfg.basis_gates.clone().unwrap_or_default(),
                optimization_level: 1,
            }
        } else {
            TranspilerConfig::default()
        };

        Ok(Self {
            name,
            client,
            job_manager,
            transpiler: Transpiler::new(transpiler_config),
            config,
            properties: None,
            calibration: None,
            runtime,
            timeout: Duration::from_secs(3600),
            use_qasm3: true,
            num_qubits,
        })
    }

    /// Create from environment variables only
    pub fn from_env(name: impl Into<String>) -> Result<Self, IbmBackendError> {
        let credentials = IbmCredentials::from_env()?;
        Self::new(name, credentials)
    }

    /// Auto-detect credentials and create backend (recommended)
    ///
    /// This tries multiple credential sources in order:
    /// 1. Environment variables (IBM_QUANTUM_TOKEN, etc.)
    /// 2. Qiskit config file (~/.qiskit/qiskit-ibm.json)
    ///
    /// Mimics Python's `QiskitRuntimeService()` behavior.
    pub fn auto_load(name: impl Into<String>) -> Result<Self, IbmBackendError> {
        let credentials = IbmCredentials::auto_load()?;
        Self::new(name, credentials)
    }

    /// Create from Qiskit config file only (~/.qiskit/qiskit-ibm.json)
    ///
    /// If `credential_name` is provided, loads that specific named credential.
    /// Otherwise, loads the default credential.
    pub fn from_qiskit_config(
        name: impl Into<String>,
        credential_name: Option<&str>,
    ) -> Result<Self, IbmBackendError> {
        let credentials = IbmCredentials::from_qiskit_config(credential_name)?;
        Self::new(name, credentials)
    }

    /// Set job timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Use QASM 2 instead of QASM 3
    pub fn with_qasm2(mut self) -> Self {
        self.use_qasm3 = false;
        self
    }

    /// Get backend configuration
    pub fn config(&self) -> Option<&BackendConfig> {
        self.config.as_ref()
    }

    /// Refresh backend properties (calibration data)
    pub fn refresh_properties(&mut self) -> Result<&BackendProperties, IbmBackendError> {
        let props = self
            .runtime
            .block_on(async { self.client.get_backend_properties(&self.name).await })?;

        self.properties = Some(props);

        // Update calibration from properties
        self.update_calibration();

        Ok(self.properties.as_ref().unwrap())
    }

    /// Update calibration from properties
    fn update_calibration(&mut self) {
        if let Some(ref props) = self.properties {
            let mut cal = CalibrationInfo::new(&self.name);

            // Extract T1, T2, errors from properties
            if let Some(ref qubits) = props.qubits {
                for (i, qubit_props) in qubits.iter().enumerate() {
                    for prop in qubit_props {
                        match prop.name.as_str() {
                            "T1" => {
                                cal.t1_times.insert(i, prop.value);
                            }
                            "T2" => {
                                cal.t2_times.insert(i, prop.value);
                            }
                            "readout_error" | "prob_meas0_prep1" => {
                                cal.readout_errors.insert(i, prop.value);
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Extract 2-qubit errors
            if let Some(ref gates) = props.gates {
                for gate in gates {
                    if (gate.gate == "cx" || gate.gate == "ecr") && gate.qubits.len() >= 2 {
                        for param in &gate.parameters {
                            if param.name == "gate_error" {
                                cal.gate_errors_2q
                                    .insert((gate.qubits[0], gate.qubits[1]), param.value);
                            }
                        }
                    }
                }
            }

            self.calibration = Some(cal);
        }
    }

    /// Get cached properties
    pub fn properties(&self) -> Option<&BackendProperties> {
        self.properties.as_ref()
    }

    /// Check if backend is operational
    pub fn is_operational(&self) -> Result<bool, IbmBackendError> {
        let status = self
            .runtime
            .block_on(async { self.client.get_backend_status(&self.name).await })?;
        Ok(status.operational)
    }

    /// Get pending jobs count
    pub fn pending_jobs(&self) -> Result<u64, IbmBackendError> {
        let status = self
            .runtime
            .block_on(async { self.client.get_backend_status(&self.name).await })?;
        Ok(status.pending_jobs.unwrap_or(0))
    }

    /// Execute circuit and wait for results
    pub fn execute_sync(
        &self,
        circuit: &Circuit,
        shots: u64,
    ) -> Result<ExecutionResult, IbmBackendError> {
        // Validate circuit
        self.transpiler
            .validate(circuit)
            .map_err(IbmBackendError::Transpilation)?;

        // Transpile to QASM
        let qasm = if self.use_qasm3 {
            self.transpiler.to_qasm3(circuit)
        } else {
            self.transpiler.to_qasm2(circuit)
        };

        // Submit job
        let result = self.runtime.block_on(async {
            let submission = JobSubmission {
                program_id: "sampler".to_string(),
                backend: self.name.clone(),
                params: JobParams::new(vec![qasm]).with_shots(shots),
                tags: vec!["niso".to_string()],
            };

            let mut job = self.job_manager.submit(submission).await?;

            // Wait for completion
            job.wait(self.timeout).await?;

            // Get results
            job.result().await
        })?;

        // Convert to ExecutionResult
        let counts = result
            .results
            .first()
            .and_then(|r| r.counts.clone())
            .unwrap_or_default();

        Ok(ExecutionResult {
            counts,
            shots,
            metadata: ExecutionMetadata {
                backend: self.name.clone(),
                job_id: result.job_id,
                simulated: false,
                ..Default::default()
            },
        })
    }

    /// Submit job asynchronously (returns job ID)
    pub fn submit_async(&self, circuit: &Circuit, shots: u64) -> Result<String, IbmBackendError> {
        // Validate and transpile
        self.transpiler
            .validate(circuit)
            .map_err(IbmBackendError::Transpilation)?;

        let qasm = if self.use_qasm3 {
            self.transpiler.to_qasm3(circuit)
        } else {
            self.transpiler.to_qasm2(circuit)
        };

        // Submit job
        let job_id = self.runtime.block_on(async {
            let submission = JobSubmission {
                program_id: "sampler".to_string(),
                backend: self.name.clone(),
                params: JobParams::new(vec![qasm]).with_shots(shots),
                tags: vec!["niso".to_string()],
            };

            let job = self.job_manager.submit(submission).await?;
            Ok::<_, JobError>(job.id().to_string())
        })?;

        Ok(job_id)
    }

    /// Get results for a submitted job
    pub fn get_results(&self, job_id: &str) -> Result<ExecutionResult, IbmBackendError> {
        let result = self.runtime.block_on(async {
            let mut job = self.job_manager.get_job(job_id).await?;

            if job.status().is_running() {
                job.wait(self.timeout).await?;
            }

            job.result().await
        })?;

        let counts = result
            .results
            .first()
            .and_then(|r| r.counts.clone())
            .unwrap_or_default();

        let shots = result.results.first().and_then(|r| r.shots).unwrap_or(0);

        Ok(ExecutionResult {
            counts,
            shots,
            metadata: ExecutionMetadata {
                backend: self.name.clone(),
                job_id: Some(job_id.to_string()),
                simulated: false,
                ..Default::default()
            },
        })
    }

    /// Execute multiple circuits in a single batch job
    pub fn execute_batch(
        &self,
        circuits: &[Circuit],
        shots: u64,
    ) -> Result<Vec<ExecutionResult>, IbmBackendError> {
        // Validate and transpile all circuits
        let qasm_circuits: Result<Vec<String>, IbmBackendError> = circuits
            .iter()
            .map(|circuit| {
                self.transpiler
                    .validate(circuit)
                    .map_err(IbmBackendError::Transpilation)?;

                Ok(if self.use_qasm3 {
                    self.transpiler.to_qasm3(circuit)
                } else {
                    self.transpiler.to_qasm2(circuit)
                })
            })
            .collect();

        let qasm_circuits = qasm_circuits?;

        // Submit batch job
        let result = self.runtime.block_on(async {
            let submission = JobSubmission {
                program_id: "sampler".to_string(),
                backend: self.name.clone(),
                params: JobParams::new(qasm_circuits).with_shots(shots),
                tags: vec!["niso".to_string(), "batch".to_string()],
            };

            let mut job = self.job_manager.submit(submission).await?;
            job.wait(self.timeout).await?;
            job.result().await
        })?;

        // Convert results
        let results: Vec<_> = result
            .results
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let counts = r.counts.clone().unwrap_or_default();
                let mut extra = std::collections::HashMap::new();
                extra.insert("circuit_index".to_string(), i.to_string());

                ExecutionResult {
                    counts,
                    shots: r.shots.unwrap_or(shots),
                    metadata: ExecutionMetadata {
                        backend: self.name.clone(),
                        job_id: result.job_id.clone(),
                        simulated: false,
                        extra,
                        ..Default::default()
                    },
                }
            })
            .collect();

        Ok(results)
    }

    /// Submit batch job asynchronously
    pub fn submit_batch_async(
        &self,
        circuits: &[Circuit],
        shots: u64,
    ) -> Result<String, IbmBackendError> {
        let qasm_circuits: Result<Vec<String>, IbmBackendError> = circuits
            .iter()
            .map(|circuit| {
                self.transpiler
                    .validate(circuit)
                    .map_err(IbmBackendError::Transpilation)?;

                Ok(if self.use_qasm3 {
                    self.transpiler.to_qasm3(circuit)
                } else {
                    self.transpiler.to_qasm2(circuit)
                })
            })
            .collect();

        let qasm_circuits = qasm_circuits?;

        let job_id = self.runtime.block_on(async {
            let submission = JobSubmission {
                program_id: "sampler".to_string(),
                backend: self.name.clone(),
                params: JobParams::new(qasm_circuits).with_shots(shots),
                tags: vec!["niso".to_string(), "batch".to_string()],
            };

            let job = self.job_manager.submit(submission).await?;
            Ok::<_, JobError>(job.id().to_string())
        })?;

        Ok(job_id)
    }
}

// Implement Backend trait for integration with NISO
impl Backend for IbmBackend {
    fn execute(&self, circuit: &Circuit, shots: u64) -> NisoResult<ExecutionResult> {
        self.execute_sync(circuit, shots)
            .map_err(|e| NisoError::BackendError(e.to_string()))
    }

    fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn calibration(&self) -> Option<&CalibrationInfo> {
        self.calibration.as_ref()
    }

    fn is_simulator(&self) -> bool {
        false
    }
}

/// List available IBM backends
pub fn list_backends(credentials: IbmCredentials) -> Result<Vec<String>, IbmBackendError> {
    let runtime = Runtime::new().map_err(|e| IbmBackendError::Runtime(e.to_string()))?;

    // Create client (async for API keys)
    let client = if credentials.requires_iam_exchange() {
        runtime.block_on(async { IbmClient::new_async(credentials).await })?
    } else {
        IbmClient::new(credentials)?
    };

    let backends = runtime.block_on(async { client.list_backends().await })?;

    Ok(backends.into_iter().map(|b| b.name).collect())
}

/// List available IBM backends using auto-detected credentials
///
/// Tries environment variables first, then ~/.qiskit/qiskit-ibm.json
pub fn list_backends_auto() -> Result<Vec<String>, IbmBackendError> {
    let credentials = IbmCredentials::auto_load()?;
    list_backends(credentials)
}

/// Get recommended backend based on job requirements
pub fn recommend_backend(
    credentials: IbmCredentials,
    min_qubits: usize,
    prefer_simulator: bool,
) -> Result<String, IbmBackendError> {
    let runtime = Runtime::new().map_err(|e| IbmBackendError::Runtime(e.to_string()))?;

    // Create client (async for API keys)
    let client = if credentials.requires_iam_exchange() {
        runtime.block_on(async { IbmClient::new_async(credentials).await })?
    } else {
        IbmClient::new(credentials)?
    };

    let backends = runtime.block_on(async { client.list_backends().await })?;

    // Filter by requirements
    let mut candidates: Vec<_> = backends
        .into_iter()
        .filter(|b| {
            let has_enough_qubits = b.num_qubits.unwrap_or(0) >= min_qubits;
            let matches_sim_pref = if prefer_simulator {
                b.simulator
            } else {
                !b.simulator
            };
            has_enough_qubits && matches_sim_pref
        })
        .collect();

    // Sort by qubit count
    candidates.sort_by_key(|b| b.num_qubits.unwrap_or(0));

    candidates.first().map(|b| b.name.clone()).ok_or_else(|| {
        IbmBackendError::NotAvailable(format!("No backend with {} qubits available", min_qubits))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use niso_core::CircuitBuilder;

    #[test]
    fn test_transpiler_integration() {
        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).measure_all().build();

        let transpiler = Transpiler::new(TranspilerConfig::default());

        assert!(transpiler.validate(&circuit).is_ok());

        let qasm = transpiler.to_qasm3(&circuit);
        assert!(qasm.contains("OPENQASM 3.0"));
    }

    // Integration tests require IBM credentials
    #[test]
    #[ignore]
    fn test_list_backends() {
        let creds = IbmCredentials::from_env().unwrap();
        let backends = list_backends(creds).unwrap();
        assert!(!backends.is_empty());
    }
}
