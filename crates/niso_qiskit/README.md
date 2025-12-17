# niso_qiskit

IBM Quantum hardware integration for NISO.

## Features

- **IBM Quantum API Integration**: Connect to IBM Quantum backends via REST API
- **QASM Transpilation**: Convert NISO circuits to OpenQASM 2.0/3.0
- **Job Management**: Submit, monitor, and retrieve quantum jobs
- **Calibration Data**: Extract T1/T2 times and error rates from hardware
- **Backend Trait**: Seamless integration with NISO optimizer

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
niso_qiskit = { path = "../niso_qiskit" }
```

## Quick Start

### Environment Setup

```bash
export IBM_QUANTUM_TOKEN="your-api-token"
export IBM_QUANTUM_INSTANCE="ibm-q/open/main"  # Optional
export IBM_QUANTUM_CHANNEL="ibm_quantum"       # Or "ibm_cloud"
```

### Basic Usage

```rust
use niso_qiskit::prelude::*;
use niso_core::CircuitBuilder;

// Create backend
let backend = IbmBackend::from_env("ibm_brisbane")?;

// Build circuit
let circuit = CircuitBuilder::new(2)
    .h(0)
    .cnot(0, 1)
    .measure_all()
    .build();

// Execute
let result = backend.execute_sync(&circuit, 4096)?;
println!("Counts: {:?}", result.counts);
```

### Async Job Submission

```rust
use niso_qiskit::prelude::*;

let backend = IbmBackend::from_env("ibm_brisbane")?;

// Submit without waiting
let job_id = backend.submit_async(&circuit, 4096)?;
println!("Job ID: {}", job_id);

// Later, retrieve results
let result = backend.get_results(&job_id)?;
```

### Using with NISO TQQC Optimizer

```rust
use niso_engine::prelude::*;
use niso_qiskit::IbmBackend;
use niso_backend::Backend;

// Create IBM backend
let ibm_backend = IbmBackend::from_env("ibm_brisbane")?;

// Check calibration data
if let Some(cal) = ibm_backend.calibration() {
    println!("T1 times: {:?}", cal.t1_times);
}

// Use Backend trait for execution
let result = ibm_backend.execute(&circuit, 4096)?;
println!("Parity: {:.4}", result.parity_expectation());
```

### Batch Circuit Execution

```rust
use niso_qiskit::prelude::*;
use niso_core::CircuitBuilder;

let backend = IbmBackend::from_env("ibm_brisbane")?;

// Create multiple circuits
let circuits: Vec<_> = (0..5).map(|i| {
    CircuitBuilder::new(7)
        .h(0)
        .rz(0, 0.1 * i as f64)
        .cnot(0, 1)
        .measure_all()
        .build()
}).collect();

// Execute all circuits in one job
let results = backend.execute_batch(&circuits, 4096)?;

for (i, result) in results.iter().enumerate() {
    println!("Circuit {}: parity = {:.4}", i, result.parity_expectation());
}
```

### List Available Backends

```rust
use niso_qiskit::prelude::*;

let creds = IbmCredentials::from_env()?;
let backends = list_backends(creds)?;
println!("Available backends: {:?}", backends);

// Recommend backend for 7 qubits
let recommended = recommend_backend(creds, 7, false)?;
println!("Recommended: {}", recommended);
```

## API Reference

### IbmBackend

| Method | Description |
|--------|-------------|
| `from_env(name)` | Create from environment variables |
| `new(name, creds)` | Create with explicit credentials |
| `execute_sync(circuit, shots)` | Execute and wait for results |
| `submit_async(circuit, shots)` | Submit without waiting |
| `get_results(job_id)` | Retrieve results for job |
| `execute_batch(circuits, shots)` | Execute multiple circuits in one job |
| `submit_batch_async(circuits, shots)` | Submit batch without waiting |
| `refresh_properties()` | Update calibration data |
| `is_operational()` | Check backend status |

### Transpiler

| Method | Description |
|--------|-------------|
| `to_qasm3(circuit)` | Convert to OpenQASM 3.0 |
| `to_qasm2(circuit)` | Convert to OpenQASM 2.0 |
| `validate(circuit)` | Validate for target backend |

### JobStatus

| Status | Terminal | Description |
|--------|----------|-------------|
| `Queued` | ❌ | Job is waiting |
| `Validating` | ❌ | Job is being validated |
| `Running` | ❌ | Job is executing |
| `Completed` | ✅ | Job finished successfully |
| `Failed` | ✅ | Job failed |
| `Cancelled` | ✅ | Job was cancelled |

## IBM Quantum Backends

### Supported Systems (2025)

| Backend | Qubits | Type |
|---------|--------|------|
| ibm_brisbane | 127 | Heron |
| ibm_sherbrooke | 127 | Eagle |
| ibm_kyiv | 127 | Eagle |
| ibm_nazca | 127 | Eagle |
| ibmq_qasm_simulator | 32 | Simulator |

### Gate Set

IBM native gates: `id`, `rz`, `sx`, `x`, `cx`, `ecr`

The transpiler automatically decomposes high-level gates to this basis set.

## Error Handling

```rust
use niso_qiskit::{IbmBackendError, ClientError, JobError};

match backend.execute_sync(&circuit, 4096) {
    Ok(result) => println!("Success: {:?}", result),
    Err(IbmBackendError::Auth(e)) => println!("Auth error: {}", e),
    Err(IbmBackendError::Job(JobError::Timeout(s))) => println!("Timeout: {}s", s),
    Err(IbmBackendError::Client(ClientError::RateLimited { retry_after })) => {
        println!("Rate limited, retry in {}s", retry_after);
    }
    Err(e) => println!("Error: {}", e),
}
```

## Architecture

```
niso_qiskit
├── auth.rs       # IBM credentials & authentication
├── client.rs     # REST API client
├── job.rs        # Job submission & monitoring
├── transpiler.rs # QASM conversion
├── backend.rs    # Backend trait implementation
└── lib.rs        # Module exports
```

## Dependencies

- `tokio`: Async runtime
- `reqwest`: HTTP client
- `serde`: JSON serialization
- `chrono`: Timestamp handling

## License

Apache-2.0
