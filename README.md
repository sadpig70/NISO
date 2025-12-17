# NISO - NISQ Integrated System Optimizer

![infographic](docs/niso_infographic.png)

High-performance Rust implementation of **TQQC (Temporal Noise Quantum Computing)** optimization algorithms for NISQ devices.

**Current Version**: 1.1 (Phase 5: IBM Hardware Integration Completed)

## Features

- **TQQC v2.2.0**: Dynamic inner loop, adaptive z-test, early stopping (~12% avg improvement).
- **IBM Quantum Integration**: Direct hardware execution via `niso_qiskit` (Native Rust implementation).
- **Multi-target Hardware**: Superconducting (IBM), Trapped Ion, Neutral Atom support.
- **Python Bindings**: Seamless Python API via `niso` package (PyO3).
- **Comprehensive Benchmarking**: Noise/qubit/points scaling analysis tools.

## Architecture

NISO follows a modular layered architecture (L0-L10):

```
NISO_System_v1.1
├── niso_core        # L0-L1: Foundation types, Circuit builder
├── niso_noise       # L2: Noise modeling (T1, T2, Errors)
├── niso_calibration # L3: Hardware calibration management
├── niso_schedule    # L4: Circuit scheduling & decoherence estimation
├── niso_tqqc        # L5: TQQC optimization engine
├── niso_backend     # L6: Execution interface (Simulator/Hardware)
├── niso_engine      # L7: High-level integration pipeline
├── niso_bench       # L8: Benchmark suites
├── niso_python      # L9: Python bindings
└── niso_qiskit      # L10: IBM Quantum Job/API management
```

## Quick Start

### 1. Robust Optimization (Simulation)

```rust
use niso_engine::prelude::*;

// Create optimizer with 7 qubits and 2% noise
let config = NisoConfig::default_7q()
    .with_noise(0.02)
    .with_seed(42);

let mut optimizer = NisoOptimizer::new(config)?;
let result = optimizer.optimize()?;

println!("Improvement: {:.2}%", result.improvement_percent());
```

### 2. IBM Quantum Real Hardware

```bash
export IBM_QUANTUM_TOKEN="your-api-token"
```

```rust
use niso_qiskit::prelude::*;
use niso_core::CircuitBuilder;

// Connect to IBM Brisbane
let backend = IbmBackend::from_env("ibm_brisbane")?;

// Execute circuit (Automatic transpilation included)
let circuit = CircuitBuilder::new(127)
    .h(0)
    .cx(0, 1)
    .measure_all()
    .build();

let result = backend.execute_sync(&circuit, 4096)?;
println!("Parity: {:.4}", result.parity_expectation());
```

### 3. Python Usage

```python
import niso

# Quick optimization
result = niso.quick_optimize(qubits=7, noise=0.02, seed=42)
print(f"Improvement: {result.improvement_percent:.2f}%")

# Generate benchmark report
suite = niso.BenchSuite(seed=42)
suite.noise_scaling(7, [0.01, 0.02, 0.03], 10)
```

## Installation

### Rust

```bash
git clone https://github.com/jungwookyang/niso.git
cd niso
cargo build --release
cargo test
```

### Python

```bash
pip install maturin
cd crates/niso_python
maturin develop --release
```

## Performance & Results

**TQQC v2.2.0** on 7-qubit H-chain (Simulation):

| Metric | Result | Note |
|--------|--------|------|
| **Avg Improvement** | **12.13%** | p=0.02 (Significant) |
| Max Improvement | 19.82% | p=0.015 |
| Computation Saved | ~51% | Early stopping & dynamic loops |

**Test Coverage (v1.1)**:

- Total Tests: **350**
- Passing: 350 (3 Ignored)
- Includes full integration tests for IBM Qiskit API.

## Project Structure

```
niso/
├── Cargo.toml          # Workspace config
├── README.md           # This file
├── docs/               # Documentation
│   ├── NISO_Unified_Specification_v1.1.md # Unified Spec
│   └── ...

└── crates/             # Source Code
    ├── niso_core/
    ├── niso_noise/
    ├── niso_calibration/
    ├── niso_schedule/
    ├── niso_tqqc/
    ├── niso_backend/
    ├── niso_engine/
    ├── niso_bench/
    ├── niso_python/
    └── niso_qiskit/    # [New] IBM Support
```

## License

MIT

## Author

**Jung Wook Yang** (<sadpig70@gmail.com>)
QC Technical Partner
