//! TQQC Optimization on IBM Quantum Hardware
//!
//! This example runs the full TQQC optimization algorithm on real hardware.
//!
//! Usage:
//! ```bash
//! # Uses saved Qiskit credentials or environment variables automatically
//! cargo run --example tqqc_optimization --release
//! ```

use niso_backend::Backend;
use niso_core::CircuitBuilder;
use niso_qiskit::prelude::*;
use std::f64::consts::PI;

/// TQQC optimization result
#[allow(dead_code)]
struct TqqcResult {
    delta_opt: f64,
    baseline_parity: f64,
    final_parity: f64,
    improvement: f64,
    improvement_percent: f64,
    iterations: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        NISO TQQC Optimization - IBM Hardware Test            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Auto-detect credentials (env vars or ~/.qiskit/qiskit-ibm.json)
    let creds = IbmCredentials::auto_load()?;
    println!("✓ IBM credentials loaded\n");

    // List backends and select
    let backends = list_backends(creds.clone())?;
    println!(
        "Available backends: {:?}\n",
        backends.iter().take(5).collect::<Vec<_>>()
    );

    // Prefer simulator for initial testing
    let backend_name = backends
        .iter()
        .find(|b| b.contains("simulator"))
        .or_else(|| backends.first())
        .cloned()
        .ok_or("No backends available")?;

    println!("Using backend: {}\n", backend_name);
    let backend = IbmBackend::new(&backend_name, creds)?;

    // Run TQQC tests at different noise levels (controlled by circuit depth)
    let test_cases = vec![
        ("5-qubit shallow", 5, 1),
        ("5-qubit medium", 5, 3),
        ("7-qubit shallow", 7, 1),
        ("7-qubit medium", 7, 2),
    ];

    println!("Running TQQC optimization tests...\n");
    println!("┌────────────────────┬────────┬──────────┬──────────┬───────────┐");
    println!("│ Test               │ Qubits │ Baseline │ Final    │ Improve   │");
    println!("├────────────────────┼────────┼──────────┼──────────┼───────────┤");

    for (name, num_qubits, depth) in test_cases {
        match run_tqqc_test(&backend, num_qubits, depth) {
            Ok(result) => {
                println!(
                    "│ {:18} │ {:6} │ {:+.4}   │ {:+.4}   │ {:+.2}%    │",
                    name,
                    num_qubits,
                    result.baseline_parity,
                    result.final_parity,
                    result.improvement_percent
                );
            }
            Err(e) => {
                println!(
                    "│ {:18} │ {:6} │ ERROR: {:30} │",
                    name,
                    num_qubits,
                    e.to_string()
                );
            }
        }
    }

    println!("└────────────────────┴────────┴──────────┴──────────┴───────────┘");
    println!("\n✓ TQQC optimization tests complete!");

    Ok(())
}

fn run_tqqc_test(
    backend: &IbmBackend,
    num_qubits: usize,
    depth: usize,
) -> Result<TqqcResult, Box<dyn std::error::Error>> {
    let shots = 4096u64;
    let theta = PI / 4.0; // Initial theta for parity circuit

    // Build parity measurement circuit
    fn build_parity_circuit(
        num_qubits: usize,
        theta: f64,
        delta: f64,
        depth: usize,
    ) -> niso_core::Circuit {
        let mut builder = CircuitBuilder::new(num_qubits);

        // Initial superposition
        builder = builder.h(0);

        // Entanglement layers (depth controls noise accumulation)
        for _ in 0..depth {
            for i in 0..(num_qubits - 1) {
                builder = builder.cnot(i, i + 1);
            }
        }

        // Rotation for parity optimization
        builder = builder.rz(0, theta + delta);

        // Measure
        builder.measure_all().build()
    }

    // Baseline measurement (delta = 0)
    let baseline_circuit = build_parity_circuit(num_qubits, theta, 0.0, depth);
    let baseline_result = backend.execute(&baseline_circuit, shots)?;
    let baseline_parity = baseline_result.parity_expectation();

    // Simple gradient search for optimal delta
    let mut best_delta = 0.0;
    let mut best_parity = baseline_parity.abs();
    let mut iterations = 0;

    let deltas = [-0.3, -0.2, -0.1, -0.05, 0.0, 0.05, 0.1, 0.2, 0.3];

    for &delta in &deltas {
        let circuit = build_parity_circuit(num_qubits, theta, delta, depth);
        let result = backend.execute(&circuit, shots)?;
        let parity = result.parity_expectation().abs();
        iterations += 1;

        if parity > best_parity {
            best_parity = parity;
            best_delta = delta;
        }
    }

    // Final measurement with optimal delta
    let final_circuit = build_parity_circuit(num_qubits, theta, best_delta, depth);
    let final_result = backend.execute(&final_circuit, shots)?;
    let final_parity = final_result.parity_expectation();

    let improvement = final_parity.abs() - baseline_parity.abs();
    let improvement_percent = if baseline_parity.abs() > 0.001 {
        (improvement / baseline_parity.abs()) * 100.0
    } else {
        0.0
    };

    Ok(TqqcResult {
        delta_opt: best_delta,
        baseline_parity,
        final_parity,
        improvement,
        improvement_percent,
        iterations,
    })
}
