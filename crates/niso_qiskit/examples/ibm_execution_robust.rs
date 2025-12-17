//! Robust IBM Quantum Execution for NISO
//!
//! Features:
//! - Hardcoded credentials (as requested)
//! - Fire-and-forget submission (--submit)
//! - Persistent job tracking (active_jobs.json)
//! - Status monitoring (--monitor)
//! - Result retrieval and analysis
//!
//! Usage:
//!   cargo run --example ibm_execution_robust -- --submit
//!   cargo run --example ibm_execution_robust -- --monitor

use niso_core::CircuitBuilder;
use niso_qiskit::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// ============================================================================
// 1. Configuration & Credentials
// ============================================================================

// SECURITY NOTE: Hardcoded credentials as per user request
const IBM_API_KEY: &str = "Fe6O7Uk_hccNJ9P4gI1PjrSjiaJ_Hd2wXGOCMK1PxG9a";
const IBM_CRN: &str = "crn:v1:bluemix:public:quantum-computing:us-east:a/81a3ca8cfbdd4b9b97f558485923bb5e:4acd9544-0813-4102-af75-bdcf82075794::";

const JOB_FILE: &str = "active_jobs.json";

// ============================================================================
// 2. Data Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
struct JobRecord {
    job_id: String,
    backend: String,
    status: String,
    submitted_at: String,
    description: String,
    results_saved: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct JobRegistry {
    jobs: Vec<JobRecord>,
}

impl JobRegistry {
    fn load() -> Self {
        if Path::new(JOB_FILE).exists() {
            let content = fs::read_to_string(JOB_FILE).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    fn save(&self) {
        let content = serde_json::to_string_pretty(self).unwrap();
        fs::write(JOB_FILE, content).unwrap();
    }

    fn add(&mut self, record: JobRecord) {
        self.jobs.push(record);
        self.save();
    }

    fn update_status(&mut self, job_id: &str, status: &str) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.job_id == job_id) {
            job.status = status.to_string();
            self.save();
        }
    }

    fn mark_saved(&mut self, job_id: &str) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.job_id == job_id) {
            job.results_saved = true;
            self.save();
        }
    }
}

// ============================================================================
// 3. Main Logic
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       NISO - Robust IBM Quantum Execution System             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let args: Vec<String> = std::env::args().collect();
    let mode = if args.contains(&"--monitor".to_string()) {
        "monitor"
    } else if args.contains(&"--submit".to_string()) {
        "submit"
    } else {
        println!("Usage:");
        println!("  cargo run --example ibm_execution_robust -- --submit   (Submit new jobs)");
        println!(
            "  cargo run --example ibm_execution_robust -- --monitor  (Check status & get results)"
        );
        return Ok(());
    };

    // Initialize credentials
    println!("▶ Initializing credentials...");
    let creds = IbmCredentials::new(format!("ApiKey-{}", IBM_API_KEY))
        .with_crn(IBM_CRN)
        .with_channel(niso_qiskit::auth::IbmChannel::IbmCloud);

    if mode == "submit" {
        // Run submit_jobs in a blocking task because it uses synchronous IbmBackend
        let creds_clone = creds.clone();
        let result = tokio::task::spawn_blocking(move || submit_jobs(creds_clone)).await;

        match result {
            Ok(inner_result) => {
                if let Err(e) = inner_result {
                    // Convert Box<dyn Error + Send + Sync> to Box<dyn Error>
                    let err: Box<dyn std::error::Error> = e;
                    return Err(err);
                }
            }
            Err(e) => return Err(Box::new(e)),
        }
    } else {
        monitor_jobs(creds).await?;
    }

    Ok(())
}

fn submit_jobs(creds: IbmCredentials) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n▶ Mode: SUBMIT");

    // 1. Select Backend
    println!("  Finding best available backend...");
    // list_backends is synchronous
    let backends = niso_qiskit::backend::list_backends(creds.clone())?;

    let mut target_backend_name = String::new();
    let mut min_pending = usize::MAX;

    let candidates = vec!["ibm_fez", "ibm_brisbane", "ibm_osaka", "ibm_kyoto"];

    for name in candidates {
        if backends.contains(&name.to_string()) {
            let backend = IbmBackend::new(name, creds.clone())?;
            // is_operational is synchronous
            if let Ok(true) = backend.is_operational() {
                // pending_jobs is synchronous
                if let Ok(pending) = backend.pending_jobs() {
                    println!("    • {} : {} pending jobs", name, pending);
                    if (pending as usize) < min_pending {
                        min_pending = pending as usize;
                        target_backend_name = name.to_string();
                    }
                }
            }
        }
    }

    if target_backend_name.is_empty() {
        // Fallback to first available
        target_backend_name = backends.first().ok_or("No backends found")?.clone();
        println!(
            "    ⚠ Could not check queue depths, falling back to: {}",
            target_backend_name
        );
    } else {
        println!(
            "    ✓ Selected: {} (Queue: {})",
            target_backend_name, min_pending
        );
    }

    let backend = IbmBackend::new(&target_backend_name, creds)?;

    // 2. Create Circuits (TQQC Verification)
    println!("  Creating verification circuits...");
    let circuits = vec![
        // 1. Bell State
        CircuitBuilder::new(2).h(0).cnot(0, 1).measure_all().build(),
        // 2. GHZ-3
        CircuitBuilder::new(3)
            .h(0)
            .cnot(0, 1)
            .cnot(1, 2)
            .measure_all()
            .build(),
    ];

    let descriptions = vec!["Bell State", "GHZ-3"];

    // 3. Submit
    let mut registry = JobRegistry::load();

    for (i, circuit) in circuits.iter().enumerate() {
        println!("  Submitting circuit {}/{}...", i + 1, circuits.len());

        // submit_async is synchronous (blocks on internal runtime)
        let job_id = backend.submit_async(circuit, 4096)?;

        println!("    ✓ Submitted! Job ID: {}", job_id);

        registry.add(JobRecord {
            job_id: job_id.clone(),
            backend: target_backend_name.clone(),
            status: "QUEUED".to_string(),
            submitted_at: chrono::Utc::now().to_rfc3339(),
            description: descriptions[i].to_string(),
            results_saved: false,
        });
    }

    println!("\n✓ All jobs submitted successfully.");
    println!("  Run 'cargo run --example ibm_execution_robust -- --monitor' to check status.");

    Ok(())
}

async fn monitor_jobs(creds: IbmCredentials) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n▶ Mode: MONITOR");

    let mut registry = JobRegistry::load();
    let active_jobs: Vec<JobRecord> = registry
        .jobs
        .iter()
        .filter(|j| !j.results_saved)
        .cloned()
        .collect();

    if active_jobs.is_empty() {
        println!("  No active jobs found in {}.", JOB_FILE);
        return Ok(());
    }

    println!("  Found {} active jobs.", active_jobs.len());
    let client = niso_qiskit::client::IbmClient::new(creds)?;
    let job_manager = niso_qiskit::job::JobManager::new(client);

    for record in active_jobs {
        println!(
            "\n  Checking Job: {} ({})",
            record.job_id, record.description
        );

        match job_manager.get_job(&record.job_id).await {
            Ok(mut job) => {
                let status_result = job.refresh().await;

                let status = match status_result {
                    Ok(s) => s,
                    Err(e) => {
                        println!("    ✗ Error refreshing job: {}", e);
                        if e.to_string().contains("Job execution failed") {
                            registry.update_status(&record.job_id, "Failed");
                            registry.mark_saved(&record.job_id);
                        }
                        continue;
                    }
                };

                println!("    Status: {:?}", status);
                registry.update_status(&record.job_id, &format!("{:?}", status));

                if status == niso_qiskit::job::JobStatus::Completed {
                    println!("    ✓ Job Completed! Downloading results...");
                    match job.result().await {
                        Ok(result) => {
                            // Analyze result
                            if let Some(circuit_res) = result.results.first() {
                                if let Some(counts) = &circuit_res.counts {
                                    println!("      Counts: {} unique bitstrings", counts.len());
                                    // Calculate Parity
                                    let total: u64 = counts.values().sum();
                                    let even: u64 = counts
                                        .iter()
                                        .filter(|(k, _)| {
                                            k.chars().filter(|&c| c == '1').count() % 2 == 0
                                        })
                                        .map(|(_, v)| v)
                                        .sum();
                                    let p_even = even as f64 / total as f64;
                                    let parity = 2.0 * p_even - 1.0;

                                    println!("      Parity: {:.4}", parity);

                                    // Save to file
                                    let filename = format!("result_{}.json", record.job_id);
                                    let json = serde_json::to_string_pretty(&result)?;
                                    fs::write(&filename, json)?;
                                    println!("      Saved to {}", filename);

                                    registry.mark_saved(&record.job_id);
                                }
                            }
                        }
                        Err(e) => println!("      ✗ Failed to get result: {}", e),
                    }
                } else if status == niso_qiskit::job::JobStatus::Failed {
                    println!("    ✗ Job Failed on backend (Status check).");
                    registry.mark_saved(&record.job_id); // Mark as done to stop checking
                }
            }
            Err(e) => println!("    ⚠ Error checking status: {}", e),
        }
    }

    println!("\n✓ Monitor cycle complete.");
    Ok(())
}
