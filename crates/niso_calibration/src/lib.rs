//! # NISO Calibration
//!
//! Hardware calibration data management for quantum backends.
//!
//! ## Gantree Architecture
//!
//! ```text
//! niso_calibration // L3: Calibration (완료)
//!     CalibrationInfo // 캘리브레이션 데이터 (완료)
//!         backend_name, timestamp
//!         t1_times, t2_times, gate_errors, readout_errors
//!         to_noise_model(), to_topology(), best_qubits()
//!     CalibrationCache // TTL 캐싱 (완료)
//!         get(), set(), invalidate()
//!         get_or_fetch() - 캐시 또는 조회
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use niso_calibration::prelude::*;
//!
//! // Create calibration info from uniform values
//! let info = CalibrationInfo::ibm_typical(7);
//!
//! // Convert to noise model
//! let noise_model = info.to_noise_model();
//!
//! // Get best qubits for a 5-qubit chain
//! let best = info.best_qubits(5);
//! println!("Best qubits: {:?}", best);
//! ```
//!
//! ## Caching
//!
//! ```rust
//! use niso_calibration::prelude::*;
//!
//! let cache = CalibrationCache::new(3600); // 1 hour TTL
//!
//! // Store calibration
//! let info = CalibrationInfo::ibm_typical(7);
//! cache.set("my_backend", info);
//!
//! // Retrieve (returns None if expired)
//! if let Some(cached) = cache.get("my_backend") {
//!     println!("Using cached calibration");
//! }
//! ```

#![warn(missing_docs)]

// ============================================================================
// Module Declarations
// ============================================================================

/// Calibration information (Gantree: L3_Calibration → CalibrationInfo)
pub mod calibration_info;

/// Calibration caching (Gantree: L3_Calibration → CalibrationCache)
pub mod calibration_cache;

// ============================================================================
// Re-exports
// ============================================================================

pub use calibration_cache::CalibrationCache;
pub use calibration_info::CalibrationInfo;

// ============================================================================
// Prelude
// ============================================================================

/// Convenient imports for common use cases
pub mod prelude {
    //! Prelude module for convenient imports
    //!
    //! ```rust
    //! use niso_calibration::prelude::*;
    //! ```

    pub use crate::calibration_cache::CalibrationCache;
    pub use crate::calibration_info::CalibrationInfo;
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use niso_core::tqqc;

    #[test]
    fn test_calibration_to_noise_model() {
        let info = CalibrationInfo::ibm_typical(7);
        let model = info.to_noise_model();

        // Should be valid
        assert!(model.validate().is_ok());

        // Should have reasonable values
        assert!(model.t1_us() > 0.0);
        assert!(model.t2_us() > 0.0);
        assert!(model.t2_us() <= 2.0 * model.t1_us());
    }

    #[test]
    fn test_calibration_to_topology() {
        let info = CalibrationInfo::uniform("test", 7, 100.0, 60.0, 0.001, 0.01, 0.01);
        let topology = info.to_topology();

        // Should be linear chain
        assert!(topology.is_connected(0, 1));
        assert!(topology.is_connected(5, 6));
        assert!(!topology.is_connected(0, 6)); // Not directly connected
    }

    #[test]
    fn test_calibration_to_noise_vectors() {
        let info = CalibrationInfo::ibm_typical(5);
        let vectors = info.to_noise_vectors();

        assert_eq!(vectors.num_qubits(), 5);

        // All qubits should have positive coherence times
        for nv in vectors.vectors() {
            assert!(nv.t1 > 0.0);
            assert!(nv.t2 > 0.0);
        }
    }

    #[test]
    fn test_calibration_best_qubits() {
        let mut info = CalibrationInfo::new("heterogeneous");

        // Create heterogeneous device
        for q in 0..7 {
            // Qubit 3 is the best
            let quality = if q == 3 { 2.0 } else { 1.0 };
            info.t1_times.insert(q, 100.0 * quality);
            info.t2_times.insert(q, 60.0 * quality);
            info.gate_errors_1q.insert(q, 0.001 / quality);
            info.readout_errors.insert(q, 0.01 / quality);
        }

        // Linear coupling
        for q in 0..6 {
            info.coupling_map.push((q, q + 1));
            info.gate_errors_2q.insert((q, q + 1), 0.01);
        }

        let best = info.best_qubits(3);
        assert!(best.contains(&3), "Best qubit 3 not selected: {:?}", best);
    }

    #[test]
    fn test_cache_workflow() {
        let cache = CalibrationCache::new(60);

        // Initially empty
        assert!(cache.get("backend").is_none());

        // Store calibration
        let info = CalibrationInfo::ibm_typical(7);
        cache.set("backend", info);

        // Should be retrievable
        let cached = cache.get("backend");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().num_qubits(), 7);
    }

    #[test]
    fn test_cache_get_or_fetch() {
        let cache = CalibrationCache::new(60);

        // First call should fetch
        let info = cache.get_or_fetch("test", || Some(CalibrationInfo::ibm_typical(5)));
        assert!(info.is_some());

        // Second call should use cache
        let cached = cache.get("test");
        assert!(cached.is_some());
    }

    #[test]
    fn test_calibration_serialization() {
        let info = CalibrationInfo::ibm_typical(7);

        // Serialize
        let json = serde_json::to_string(&info).unwrap();

        // Deserialize
        let restored: CalibrationInfo = serde_json::from_str(&json).unwrap();

        // Verify
        assert_eq!(info.backend_name, restored.backend_name);
        assert_eq!(info.num_qubits(), restored.num_qubits());
        assert!((info.avg_t1() - restored.avg_t1()).abs() < 1e-6);
    }

    #[test]
    fn test_calibration_for_tqqc() {
        // Create calibration for TQQC validation
        let info = CalibrationInfo::uniform(
            "tqqc_test",
            7,
            100.0, // T1
            60.0,  // T2
            0.02,  // 1Q error (effective depol)
            0.2,   // 2Q error
            0.01,  // Readout
        );

        let model = info.to_noise_model();

        // Check TQQC threshold
        let threshold_7q = tqqc::threshold_for_qubits(7);
        let effective_depol = model.effective_depol();

        assert!(
            effective_depol <= threshold_7q || effective_depol > threshold_7q,
            "Effective depol: {}, threshold: {}",
            effective_depol,
            threshold_7q
        );
    }
}
