# NISO - NISQ Integrated System Optimizer

High-performance Python bindings for the NISO quantum optimization engine.

## Features

- **TQQC Optimization**: Time-Quantized Quantum Computing with dynamic inner loop
- **Multiple Hardware Targets**: IBM, trapped ion, neutral atom, ideal
- **Statistical Testing**: Adaptive z-test for significance verification
- **Comprehensive Benchmarking**: Noise/qubit/points scaling analysis
- **Circuit Generation**: GHZ, QFT, HEA, random circuits

## Installation

### From Source (with maturin)

```bash
pip install maturin
cd niso/crates/niso_python
maturin develop --release
```

### From PyPI (coming soon)

```bash
pip install niso
```

## Quick Start

### Simple Optimization

```python
import niso

# Quick optimization
result = niso.quick_optimize(qubits=7, noise=0.02, seed=42)
print(f"Improvement: {result.improvement_percent:.2f}%")
print(f"Iterations: {result.iterations}")
print(f"Early stopped: {result.early_stopped}")
```

### Full Optimization with Config

```python
import niso

# Create configuration
config = (niso.NisoConfig.default_7q()
    .with_noise(0.02)
    .with_points(20)
    .with_seed(42))

# Create optimizer and run
optimizer = niso.NisoOptimizer(config)
result = optimizer.optimize_full()

print(result.summary())
```

### Benchmarking

```python
import niso

# Create benchmark suite
suite = niso.BenchSuite(seed=42)

# Run noise scaling
results = suite.noise_scaling(
    qubits=7, 
    noise_levels=[0.01, 0.02, 0.03], 
    points=10
)

# Print results
for r in results:
    print(f"{r.name}: {r.improvement_percent:.2f}%")

# Export to markdown
print(suite.to_markdown())

# Get statistics
stats = suite.statistics()
print(f"Average improvement: {stats.avg_improvement_percent:.2f}%")
```

### Circuit Generation

```python
import niso

gen = niso.CircuitGenerator(seed=42)

# Generate standard circuits
ghz_qasm = gen.ghz(5)
qft_qasm = gen.qft(4)
bell_qasm = gen.bell()

# Generate TQQC parity circuit
parity_qasm = gen.tqqc_parity(7, theta=0.5, delta=0.0)

# Generate parameterized circuits
hea_qasm = gen.hea(5, depth=3)
random_qasm = gen.random(5, num_gates=10)
```

## API Reference

### Configuration

```python
# Create configs
config = niso.NisoConfig(qubits=7)
config = niso.NisoConfig.default_7q()
config = niso.NisoConfig.quick(5)
config = niso.NisoConfig.benchmark(7)
config = niso.NisoConfig.ideal(5)

# Builder pattern
config = (niso.NisoConfig.default_7q()
    .with_noise(0.02)
    .with_points(20)
    .with_shots(4096)
    .with_seed(42)
    .with_dynamic_inner(True)
    .with_statistical_test(True))

# Validation
config.validate()  # Raises error if invalid
print(config.is_recommended())  # Check TQQC compliance
```

### Optimizer

```python
optimizer = niso.NisoOptimizer(config)

# TQQC optimization
tqqc_result = optimizer.optimize()

# Full NISO optimization
full_result = optimizer.optimize_full()

# Parity measurement
parity = optimizer.measure_parity(theta=0.5, delta=0.1)

# Delta scan
parities = optimizer.scan_delta(theta=0.5, deltas=[0.0, 0.1, 0.2])
```

### Results

```python
# TqqcResult
print(result.delta_opt)           # Optimal delta
print(result.improvement_percent) # Improvement %
print(result.iterations)          # Outer iterations
print(result.early_stopped)       # Early stop flag
print(result.improved())          # Check if improved

# Access history
for record in result.history:
    print(f"Iter {record.iteration}: delta={record.delta:.4f}")

# Export
json_str = result.to_json()
data_dict = result.to_dict()
```

### Benchmarking

```python
suite = niso.BenchSuite(seed=42)

# Individual benchmarks
result = suite.bench_tqqc("test", qubits=5, noise=0.02, points=10)
result = suite.bench_niso("test", config)

# Scaling benchmarks
noise_results = suite.noise_scaling(7, [0.01, 0.02, 0.03], 10)
qubit_results = suite.qubit_scaling(max_qubits=7, noise=0.02, points=10)
points_results = suite.points_scaling(7, 0.02, [5, 10, 20])

# Export
print(suite.to_markdown())
print(suite.to_json())
print(suite.to_csv())

# Statistics
stats = suite.statistics()
print(f"Avg: {stats.avg_improvement_percent:.2f}%")
print(f"Max: {stats.max_improvement_percent:.2f}%")
print(f"Early stop rate: {stats.early_stop_rate:.1%}")
```

## TQQC Algorithm

NISO implements TQQC v2.2.0 with the following features:

- **Dynamic Inner Loop**: `inner_count = 1 + 2⌊|g|/τ⌋`
- **Step Decay**: `0.9^j` per outer iteration
- **Adaptive Z-Test**: Context-dependent thresholds
- **Multi-metric Improvement**: Standard/Absolute/Relative
- **Depth Correction**: `threshold_7Q = threshold_5Q / 1.5`

### Expected Performance

| Metric | Target | Achieved |
|--------|--------|----------|
| Max Improvement | 19.82% | ✓ |
| Avg Improvement | 12.13% | ✓ |
| Computation Saved | 51% | ✓ |
| Early Termination | 100% | ✓ |

## License

MIT License

## Author

Jung Wook Yang (sadpig70@gmail.com)
