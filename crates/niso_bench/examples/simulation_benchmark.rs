//! NISO TQQC Simulation Benchmark
//!
//! Runs comprehensive benchmarks using the simulator backend.
//!
//! NOTE: TQQC optimizes for COHERENT errors (systematic rotations),
//! not INCOHERENT errors (depolarizing noise). This benchmark
//! simulates coherent errors to demonstrate TQQC effectiveness.

use niso_backend::{Backend, SimulatorBackend};
use niso_core::CircuitBuilder;
use std::f64::consts::PI;
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║              NISO TQQC Simulation Benchmark Report                   ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    println!("NOTE: TQQC optimizes COHERENT errors (systematic rotations).");
    println!("      This benchmark simulates coherent gate errors.\n");

    // Benchmark configurations
    let qubit_configs = vec![3, 5, 7];
    let coherent_errors = vec![0.02, 0.05, 0.10, 0.15]; // rad
    let shots = 4096u64;
    let seed = 42u64;

    println!("Configuration:");
    println!("  • Qubits: {:?}", qubit_configs);
    println!("  • Coherent errors: {:?} rad", coherent_errors);
    println!("  • Shots per circuit: {}", shots);
    println!("  • Random seed: {}", seed);
    println!();

    // Results storage
    let mut all_results: Vec<BenchResult> = Vec::new();

    // =========================================================================
    // Benchmark 1: Coherent Error Scaling
    // =========================================================================
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 1: Coherent Error Scaling (7-qubit)");
    println!("═══════════════════════════════════════════════════════════════════════\n");

    println!("┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐");
    println!("│ Error(r) │ Baseline │ Final    │ Improve  │ Iters    │ Time(ms) │");
    println!("├──────────┼──────────┼──────────┼──────────┼──────────┼──────────┤");

    for &error in &coherent_errors {
        let result = run_tqqc_benchmark(7, error, 0.01, shots, seed);
        println!(
            "│ {:.3}    │ {:+.4}   │ {:+.4}   │ {:+6.2}%  │ {:8} │ {:8.1} │",
            error,
            result.baseline_parity,
            result.final_parity,
            result.improvement_percent,
            result.iterations,
            result.time_ms
        );
        all_results.push(result);
    }

    println!("└──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘\n");

    // =========================================================================
    // Benchmark 2: Qubit Scaling
    // =========================================================================
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 2: Qubit Scaling (coherent_error=0.10 rad)");
    println!("═══════════════════════════════════════════════════════════════════════\n");

    println!("┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐");
    println!("│ Qubits   │ Baseline │ Final    │ Improve  │ Iters    │ Time(ms) │");
    println!("├──────────┼──────────┼──────────┼──────────┼──────────┼──────────┤");

    for &qubits in &qubit_configs {
        let result = run_tqqc_benchmark(qubits, 0.10, 0.01, shots, seed);
        println!(
            "│ {:8} │ {:+.4}   │ {:+.4}   │ {:+6.2}%  │ {:8} │ {:8.1} │",
            qubits,
            result.baseline_parity,
            result.final_parity,
            result.improvement_percent,
            result.iterations,
            result.time_ms
        );
        all_results.push(result);
    }

    println!("└──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘\n");

    // =========================================================================
    // Benchmark 3: TQQC v2.2.0 Parameters Verification
    // =========================================================================
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 3: TQQC v2.2.0 Algorithm Verification");
    println!("═══════════════════════════════════════════════════════════════════════\n");

    let test_cases = vec![
        ("5Q, small err", 5, 0.05),
        ("5Q, med err", 5, 0.10),
        ("5Q, large err", 5, 0.15),
        ("7Q, small err", 7, 0.05),
        ("7Q, med err", 7, 0.10),
        ("7Q, large err", 7, 0.15),
    ];

    println!("┌────────────────┬────────┬───────┬──────────┬──────────┬──────────┐");
    println!("│ Test Case      │ Qubits │ Error │ Baseline │ Final    │ Improve  │");
    println!("├────────────────┼────────┼───────┼──────────┼──────────┼──────────┤");

    for (name, qubits, error) in test_cases {
        let result = run_tqqc_benchmark(qubits, error, 0.01, shots, seed);
        println!(
            "│ {:14} │ {:6} │ {:.3} │ {:+.4}   │ {:+.4}   │ {:+6.2}%  │",
            name,
            qubits,
            error,
            result.baseline_parity,
            result.final_parity,
            result.improvement_percent
        );
        all_results.push(result);
    }

    println!("└────────────────┴────────┴───────┴──────────┴──────────┴──────────┘\n");

    // =========================================================================
    // Benchmark 4: Statistical Consistency (Multiple Runs)
    // =========================================================================
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 4: Statistical Consistency (10 runs, 7Q, error=0.10)");
    println!("═══════════════════════════════════════════════════════════════════════\n");

    let mut improvements: Vec<f64> = Vec::new();
    let mut baselines: Vec<f64> = Vec::new();
    let mut finals: Vec<f64> = Vec::new();

    for run in 0..10 {
        let result = run_tqqc_benchmark(7, 0.10, 0.01, shots, seed + run as u64);
        improvements.push(result.improvement_percent);
        baselines.push(result.baseline_parity);
        finals.push(result.final_parity);
    }

    let avg_improvement = improvements.iter().sum::<f64>() / improvements.len() as f64;
    let max_improvement = improvements
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let min_improvement = improvements.iter().cloned().fold(f64::INFINITY, f64::min);
    let std_dev = {
        let variance = improvements
            .iter()
            .map(|x| (x - avg_improvement).powi(2))
            .sum::<f64>()
            / improvements.len() as f64;
        variance.sqrt()
    };

    println!("  Improvement Statistics:");
    println!("    • Average:  {:+.2}%", avg_improvement);
    println!("    • Maximum:  {:+.2}%", max_improvement);
    println!("    • Minimum:  {:+.2}%", min_improvement);
    println!("    • Std Dev:  {:.2}%", std_dev);
    println!();

    let avg_baseline = baselines.iter().sum::<f64>() / baselines.len() as f64;
    let avg_final = finals.iter().sum::<f64>() / finals.len() as f64;

    println!("  Parity Statistics:");
    println!("    • Avg Baseline: {:+.4}", avg_baseline);
    println!("    • Avg Final:    {:+.4}", avg_final);
    println!();

    // =========================================================================
    // Summary
    // =========================================================================
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("  SUMMARY");
    println!("═══════════════════════════════════════════════════════════════════════\n");

    let total_improvements: Vec<f64> = all_results.iter().map(|r| r.improvement_percent).collect();
    let overall_avg = total_improvements.iter().sum::<f64>() / total_improvements.len() as f64;
    let overall_max = total_improvements
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let positive_count = total_improvements.iter().filter(|&&x| x > 0.0).count();

    println!("  Total benchmarks run: {}", all_results.len());
    println!("  Average improvement:  {:+.2}%", overall_avg);
    println!("  Maximum improvement:  {:+.2}%", overall_max);
    println!(
        "  Positive improvements: {}/{} ({:.1}%)",
        positive_count,
        all_results.len(),
        (positive_count as f64 / all_results.len() as f64) * 100.0
    );
    println!();

    // TQQC v2.2.0 target comparison
    println!("  TQQC v2.2.0 Target Comparison:");
    println!("    • Target max improvement: 19.82%");
    println!("    • Target avg improvement: 12.13%");
    println!("    • Achieved max:           {:+.2}%", overall_max);
    println!("    • Achieved avg:           {:+.2}%", overall_avg);
    println!();

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                    Benchmark Complete ✓                              ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
}

#[allow(dead_code)]
struct BenchResult {
    qubits: usize,
    coherent_error: f64,
    noise: f64,
    baseline_parity: f64,
    final_parity: f64,
    improvement_percent: f64,
    iterations: usize,
    time_ms: f64,
    optimal_delta: f64,
}

fn run_tqqc_benchmark(
    num_qubits: usize,
    coherent_error: f64,
    noise: f64,
    shots: u64,
    seed: u64,
) -> BenchResult {
    let start = Instant::now();

    // Create simulator backend with depolarizing noise
    let backend = SimulatorBackend::from_depol(num_qubits, noise)
        .expect("Failed to create backend")
        .with_seed(seed);

    let theta = PI / 4.0;

    // Build TQQC parity circuit with COHERENT ERROR
    //
    // TQQC Parity Circuit Structure:
    // 1. GHZ preparation: H(0), CNOT chain
    // 2. Phase encoding: Rz(theta + coherent_error + delta) on all qubits
    // 3. Inverse GHZ: CNOT chain (reverse), H(0)
    // 4. Measurement: All qubits
    //
    // This creates interference pattern where parity depends on total phase
    let build_circuit = |delta: f64| -> niso_core::Circuit {
        let mut builder = CircuitBuilder::new(num_qubits);

        // 1. GHZ preparation
        builder = builder.h(0);
        for i in 0..(num_qubits - 1) {
            builder = builder.cnot(i, i + 1);
        }

        // 2. Phase rotation on ALL qubits
        // Hardware has coherent error, TQQC compensates with delta
        let actual_phase = theta + coherent_error + delta;
        for i in 0..num_qubits {
            builder = builder.rz(i, actual_phase / num_qubits as f64);
        }

        // 3. Inverse GHZ (creates interference)
        for i in (0..(num_qubits - 1)).rev() {
            builder = builder.cnot(i, i + 1);
        }
        builder = builder.h(0);

        // 4. Measure
        builder.measure_all().build()
    };

    // Baseline measurement (delta = 0, no TQQC compensation)
    // With coherent_error, this should give suboptimal parity
    let baseline_circuit = build_circuit(0.0);
    let baseline_result = backend.execute(&baseline_circuit, shots).unwrap();
    let baseline_parity = baseline_result.parity_expectation();

    // TQQC optimization: search for optimal delta
    // Ideal delta should be approximately -coherent_error
    let mut best_delta = 0.0;
    let mut best_parity = baseline_parity.abs();
    let mut iterations = 0;

    // Coarse search: [-0.5, 0.5] rad
    let coarse_deltas: Vec<f64> = (-10..=10).map(|i| i as f64 * 0.05).collect();
    for &delta in &coarse_deltas {
        let circuit = build_circuit(delta);
        let result = backend.execute(&circuit, shots).unwrap();
        let parity = result.parity_expectation().abs();
        iterations += 1;

        if parity > best_parity {
            best_parity = parity;
            best_delta = delta;
        }
    }

    // Fine search around best delta
    let fine_range: Vec<f64> = (-5..=5).map(|i| i as f64 * 0.01).collect();
    for &offset in &fine_range {
        let delta = best_delta + offset;
        let circuit = build_circuit(delta);
        let result = backend.execute(&circuit, shots).unwrap();
        let parity = result.parity_expectation().abs();
        iterations += 1;

        if parity > best_parity {
            best_parity = parity;
            best_delta = delta;
        }
    }

    // Final measurement with optimal delta
    let final_circuit = build_circuit(best_delta);
    let final_result = backend.execute(&final_circuit, shots).unwrap();
    let final_parity = final_result.parity_expectation();

    let improvement = final_parity.abs() - baseline_parity.abs();
    let improvement_percent = if baseline_parity.abs() > 0.001 {
        (improvement / baseline_parity.abs()) * 100.0
    } else {
        // Use absolute improvement if baseline is near zero
        improvement * 100.0
    };

    let elapsed = start.elapsed();

    BenchResult {
        qubits: num_qubits,
        coherent_error,
        noise,
        baseline_parity,
        final_parity,
        improvement_percent,
        iterations,
        time_ms: elapsed.as_secs_f64() * 1000.0,
        optimal_delta: best_delta,
    }
}
