//! # NISO Qiskit
//!
//! IBM Quantum hardware integration for NISO.
//!
//! ## Gantree Architecture
//!
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use niso_qiskit::prelude::*;
//!
//! // Create backend from environment
//! let backend = IbmBackend::from_env("ibm_brisbane")?;
//!
//! // Execute circuit
//! let circuit = /* your circuit */;
//! let result = backend.execute_sync(&circuit, 4096)?;
//!
//! println!("Counts: {:?}", result.counts);
//! ```
//!
//! ## Environment Variables
//!
//! ```bash
//! export IBM_QUANTUM_TOKEN="your-api-token"
//! export IBM_QUANTUM_INSTANCE="ibm-q/open/main"  # Optional
//! export IBM_QUANTUM_CHANNEL="ibm_quantum"       # Or "ibm_cloud"
//! ```
//!
//! ## Using with NISO Engine
//!
//! ```rust,ignore
//! use niso_engine::prelude::*;
//! use niso_qiskit::IbmBackend;
//!
//! // Create IBM backend
//! let ibm_backend = IbmBackend::from_env("ibm_brisbane")?;
//!
//! // Use with NISO optimizer
//! let config = NisoConfig::default_7q()
//!     .with_noise(0.02);
//!
//! // Note: Integration with NisoOptimizer requires
//! // using the Backend trait implementation
//! ```
//!
//! ## Async Job Submission
//!
//! ```rust,ignore
//! use niso_qiskit::prelude::*;
//!
//! let backend = IbmBackend::from_env("ibm_brisbane")?;
//!
//! // Submit job without waiting
//! let job_id = backend.submit_async(&circuit, 4096)?;
//! println!("Job submitted: {}", job_id);
//!
//! // Later, retrieve results
//! let result = backend.get_results(&job_id)?;
//! ```

#![warn(missing_docs)]

// ============================================================================
// Module Declarations
// ============================================================================

/// Authentication (Gantree: L10_Qiskit ??Auth)
pub mod auth;

/// REST API client (Gantree: L10_Qiskit ??Client)
pub mod client;

/// Job management (Gantree: L10_Qiskit ??Job)
pub mod job;

/// Circuit transpilation (Gantree: L10_Qiskit ??Transpiler)
pub mod transpiler;

/// IBM backend implementation (Gantree: L10_Qiskit ??Backend)
pub mod backend;

// ============================================================================
// Re-exports
// ============================================================================

pub use auth::{AuthError, IbmChannel, IbmCredentials, TokenType};
pub use backend::{list_backends, recommend_backend, IbmBackend, IbmBackendError};
pub use client::{
    BackendConfig, BackendInfo, BackendProperties, BackendStatus, ClientError, IbmClient,
};
pub use job::{
    CircuitResult, IbmJob, JobError, JobManager, JobParams, JobResponse, JobResult, JobStatus,
    JobSubmission,
};
pub use transpiler::{Transpiler, TranspilerConfig, IBM_BASIS_GATES};

// ============================================================================
// Prelude
// ============================================================================

// Convenient imports below
/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::auth::{IbmChannel, IbmCredentials};
    pub use crate::backend::{list_backends, recommend_backend, IbmBackend};
    pub use crate::client::IbmClient;
    pub use crate::job::{JobManager, JobParams, JobStatus};
    pub use crate::transpiler::{Transpiler, TranspilerConfig};
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use niso_core::CircuitBuilder;

    #[test]
    fn test_credentials_creation() {
        let creds = IbmCredentials::new("test_token_12345678901234567890123456789012");
        assert_eq!(creds.channel(), IbmChannel::IbmQuantum);
    }

    #[test]
    fn test_transpiler_qasm3() {
        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).measure_all().build();

        let transpiler = Transpiler::new(TranspilerConfig::default());
        let qasm = transpiler.to_qasm3(&circuit);

        assert!(qasm.contains("OPENQASM 3.0"));
        assert!(qasm.contains("qubit[2]"));
    }

    #[test]
    fn test_transpiler_qasm2() {
        let circuit = CircuitBuilder::new(3)
            .h(0)
            .cnot(0, 1)
            .cnot(1, 2)
            .measure_all()
            .build();

        let transpiler = Transpiler::new(TranspilerConfig::default());
        let qasm = transpiler.to_qasm2(&circuit);

        assert!(qasm.contains("OPENQASM 2.0"));
        assert!(qasm.contains("qreg q[3]"));
        assert!(qasm.contains("creg c[3]"));
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
    fn test_job_status_checks() {
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(JobStatus::Running.is_running());
        assert!(JobStatus::Completed.is_success());
    }

    #[test]
    fn test_transpiler_validation() {
        let circuit = CircuitBuilder::new(5).h(0).cnot(0, 1).build();

        // Valid for large backend
        let config_large = TranspilerConfig {
            num_qubits: 127,
            ..Default::default()
        };
        let transpiler = Transpiler::new(config_large);
        assert!(transpiler.validate(&circuit).is_ok());

        // Invalid for small backend
        let config_small = TranspilerConfig {
            num_qubits: 3,
            ..Default::default()
        };
        let transpiler_small = Transpiler::new(config_small);
        assert!(transpiler_small.validate(&circuit).is_err());
    }

    #[test]
    fn test_channel_urls() {
        let quantum_url = IbmChannel::IbmQuantum.base_url();
        let cloud_url = IbmChannel::IbmCloud.base_url();

        // Both channels now use the unified quantum.cloud.ibm.com endpoint (2025 API)
        assert!(quantum_url.contains("quantum.cloud.ibm.com"));
        assert!(cloud_url.contains("quantum.cloud.ibm.com"));
    }

    #[test]
    fn test_auth_header() {
        let creds = IbmCredentials::new("my_token");
        assert_eq!(creds.auth_header(), "Bearer my_token");
    }

    #[test]
    fn test_backend_info_deserialize() {
        let json = r#"{
            "name": "ibm_sherbrooke",
            "n_qubits": 127,
            "simulator": false,
            "version": "2.0.0"
        }"#;

        let info: BackendInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "ibm_sherbrooke");
        assert_eq!(info.num_qubits, Some(127));
        assert!(!info.simulator);
    }

    #[test]
    fn test_circuit_result_deserialize() {
        let json = r#"{
            "counts": {"00": 500, "11": 500},
            "success": true,
            "shots": 1000
        }"#;

        let result: CircuitResult = serde_json::from_str(json).unwrap();
        assert!(result.success);
        assert_eq!(result.shots, Some(1000));

        let counts = result.counts.unwrap();
        assert_eq!(counts.len(), 2);
    }

    #[test]
    fn test_ghz_circuit_transpilation() {
        // 7-qubit GHZ state
        let mut builder = CircuitBuilder::new(7);
        builder = builder.h(0);
        for i in 0..6 {
            builder = builder.cnot(i, i + 1);
        }
        let circuit = builder.measure_all().build();

        let transpiler = Transpiler::new(TranspilerConfig::default());
        let qasm = transpiler.to_qasm3(&circuit);

        // Should have 6 CX gates
        assert_eq!(qasm.matches("cx q[").count(), 6);
    }

    #[test]
    fn test_parity_circuit_transpilation() {
        // Simple parity-like circuit
        let circuit = CircuitBuilder::new(5)
            .h(0)
            .cnot(0, 1)
            .cnot(1, 2)
            .cnot(2, 3)
            .cnot(3, 4)
            .rz(0, 0.5)
            .measure_all()
            .build();

        let transpiler = Transpiler::new(TranspilerConfig::default());

        // QASM 3
        let qasm3 = transpiler.to_qasm3(&circuit);
        assert!(qasm3.contains("rz("));

        // QASM 2
        let qasm2 = transpiler.to_qasm2(&circuit);
        assert!(qasm2.contains("rz(0.5)"));
    }
}
