"""
NISO - NISQ Integrated System Optimizer

Python bindings for high-performance quantum optimization.

Example:
    >>> import niso
    >>> result = niso.quick_optimize(qubits=7, noise=0.02)
    >>> print(f"Improvement: {result.improvement_percent:.2f}%")
"""

from typing import Optional, List, Dict, Any

__version__: str
__author__: str


class OptimizationMode:
    """Optimization mode selection."""
    
    @staticmethod
    def full() -> "OptimizationMode": ...
    @staticmethod
    def quick() -> "OptimizationMode": ...
    @staticmethod
    def benchmark() -> "OptimizationMode": ...
    @staticmethod
    def custom() -> "OptimizationMode": ...


class HardwareTarget:
    """Hardware target selection."""
    
    @staticmethod
    def ibm() -> "HardwareTarget": ...
    @staticmethod
    def trapped_ion() -> "HardwareTarget": ...
    @staticmethod
    def neutral_atom() -> "HardwareTarget": ...
    @staticmethod
    def ideal() -> "HardwareTarget": ...


class NisoConfig:
    """NISO configuration."""
    
    qubits: int
    noise: float
    points: int
    shots: int
    seed: Optional[int]
    dynamic_inner: bool
    use_statistical_test: bool
    
    def __init__(self, qubits: int = 7) -> None: ...
    
    @staticmethod
    def default_5q() -> "NisoConfig": ...
    @staticmethod
    def default_7q() -> "NisoConfig": ...
    @staticmethod
    def quick(qubits: int) -> "NisoConfig": ...
    @staticmethod
    def benchmark(qubits: int) -> "NisoConfig": ...
    @staticmethod
    def ideal(qubits: int) -> "NisoConfig": ...
    
    def with_qubits(self, qubits: int) -> "NisoConfig": ...
    def with_noise(self, noise: float) -> "NisoConfig": ...
    def with_points(self, points: int) -> "NisoConfig": ...
    def with_shots(self, shots: int) -> "NisoConfig": ...
    def with_seed(self, seed: int) -> "NisoConfig": ...
    def with_dynamic_inner(self, enabled: bool) -> "NisoConfig": ...
    def with_statistical_test(self, enabled: bool) -> "NisoConfig": ...
    def with_verbose(self, verbose: bool) -> "NisoConfig": ...
    def with_mode(self, mode: OptimizationMode) -> "NisoConfig": ...
    def with_hardware(self, hardware: HardwareTarget) -> "NisoConfig": ...
    
    def is_recommended(self) -> bool: ...
    def validate(self) -> None: ...
    def to_json(self) -> str: ...
    def to_dict(self) -> Dict[str, Any]: ...


class IterationRecord:
    """Single iteration record."""
    
    iteration: int
    delta: float
    parity_plus: float
    parity_minus: float
    parity_selected: float
    improvement: float
    inner_count: int
    direction: str
    is_significant: bool


class TqqcResult:
    """TQQC optimization result."""
    
    delta_opt: float
    parity_baseline: float
    parity_final: float
    improvement: float
    improvement_percent: float
    iterations: int
    early_stopped: bool
    ties_count: int
    significant_moves: int
    total_inner_iterations: int
    history: List[IterationRecord]
    
    def improved(self) -> bool: ...
    def k_estimated(self, max_points: int) -> float: ...
    def to_json(self) -> str: ...
    def to_dict(self) -> Dict[str, Any]: ...


class OptimizationResult:
    """Full optimization result."""
    
    tqqc_result: TqqcResult
    delta_opt: float
    baseline_parity: float
    final_parity: float
    improvement: float
    improvement_percent: float
    iterations: int
    early_stopped: bool
    total_time_ms: int
    circuit_executions: int
    total_shots: int
    
    def schedule_metrics(self) -> Optional[Dict[str, float]]: ...
    def calibration_summary(self) -> Optional[Dict[str, float]]: ...
    def to_json(self) -> str: ...
    def to_dict(self) -> Dict[str, Any]: ...
    def summary(self) -> str: ...


class NisoOptimizer:
    """NISO optimizer."""
    
    config: NisoConfig
    
    def __init__(self, config: NisoConfig) -> None: ...
    def optimize(self) -> TqqcResult: ...
    def optimize_full(self) -> OptimizationResult: ...
    def measure_parity(self, theta: float, delta: float) -> float: ...
    def scan_delta(self, theta: float, deltas: List[float]) -> Dict[str, float]: ...


class BenchmarkResult:
    """Single benchmark result."""
    
    name: str
    qubits: int
    noise: float
    baseline: float
    final_parity: float
    improvement: float
    improvement_percent: float
    iterations: int
    early_stopped: bool
    time_ms: int
    total_shots: int
    
    def to_dict(self) -> Dict[str, Any]: ...


class Statistics:
    """Benchmark statistics."""
    
    count: int
    avg_improvement_percent: float
    max_improvement_percent: float
    min_improvement_percent: float
    avg_time_ms: float
    total_time_ms: int
    early_stop_rate: float
    
    def to_dict(self) -> Dict[str, float]: ...


class BenchSuite:
    """Benchmark suite."""
    
    def __init__(self, seed: Optional[int] = None) -> None: ...
    
    def bench_tqqc(self, name: str, qubits: int, noise: float, points: int) -> BenchmarkResult: ...
    def bench_niso(self, name: str, config: NisoConfig) -> BenchmarkResult: ...
    def noise_scaling(self, qubits: int, noise_levels: List[float], points: int) -> List[BenchmarkResult]: ...
    def qubit_scaling(self, max_qubits: int, noise: float, points: int) -> List[BenchmarkResult]: ...
    def points_scaling(self, qubits: int, noise: float, point_counts: List[int]) -> List[BenchmarkResult]: ...
    def run_all(self) -> List[BenchmarkResult]: ...
    def run_quick(self) -> List[BenchmarkResult]: ...
    def results(self) -> List[BenchmarkResult]: ...
    def statistics(self) -> Statistics: ...
    def clear(self) -> None: ...
    def len(self) -> int: ...
    def is_empty(self) -> bool: ...
    def to_json(self) -> str: ...
    def to_csv(self) -> str: ...
    def to_markdown(self) -> str: ...
    def to_text(self) -> str: ...
    def __len__(self) -> int: ...


class CircuitGenerator:
    """Circuit generator."""
    
    def __init__(self, seed: Optional[int] = None) -> None: ...
    
    def ghz(self, num_qubits: int) -> str: ...
    def bell(self) -> str: ...
    def qft(self, num_qubits: int) -> str: ...
    def hea(self, num_qubits: int, depth: int) -> str: ...
    def random(self, num_qubits: int, num_gates: int) -> str: ...
    def tqqc_parity(self, num_qubits: int, theta: float, delta: float) -> str: ...
    def w_state(self, num_qubits: int) -> str: ...


def quick_optimize(
    qubits: int = 7,
    noise: float = 0.02,
    points: int = 20,
    shots: int = 4096,
    seed: Optional[int] = None
) -> TqqcResult: ...


def full_optimize(
    qubits: int = 7,
    noise: float = 0.02,
    points: int = 20,
    shots: int = 4096,
    seed: Optional[int] = None
) -> OptimizationResult: ...


def noise_scaling_benchmark(
    qubits: int = 7,
    noise_levels: Optional[List[float]] = None,
    points: int = 10
) -> List[BenchmarkResult]: ...


def qubit_scaling_benchmark(
    max_qubits: int = 7,
    noise: float = 0.02,
    points: int = 10
) -> List[BenchmarkResult]: ...
