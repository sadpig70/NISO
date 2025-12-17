//! Qubit topology for NISO
//!
//! Gantree: L1_Circuit → Topology
//!
//! Provides topology representations for qubit connectivity,
//! supporting both hardware-agnostic and hardware-specific layouts.

use crate::circuit::Circuit;
use crate::error::{NisoError, NisoResult};
use crate::types::QubitId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Qubit topology (coupling map)
/// Gantree: Topology // 큐비트 토폴로지
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Topology {
    /// Coupling map: list of (control, target) pairs
    /// Gantree: coupling_map: Vec<(QubitId, QubitId)> // 연결 맵
    coupling_map: Vec<(QubitId, QubitId)>,

    /// Number of qubits
    /// Gantree: num_qubits: usize // 큐비트 수
    num_qubits: usize,

    /// Whether topology is bidirectional
    bidirectional: bool,

    /// Optional topology name
    name: Option<String>,
}

impl Topology {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create from coupling map
    /// Gantree: from_coupling_map(map) -> Self // 맵에서 생성
    pub fn from_coupling_map(
        coupling_map: Vec<(QubitId, QubitId)>,
        bidirectional: bool,
    ) -> NisoResult<Self> {
        if coupling_map.is_empty() {
            return Err(NisoError::EmptyCouplingMap);
        }

        // Validate couplings
        for &(q1, q2) in &coupling_map {
            if q1 == q2 {
                return Err(NisoError::InvalidCoupling(q1, q2));
            }
        }

        // Find max qubit
        let max_qubit = coupling_map
            .iter()
            .flat_map(|&(a, b)| vec![a, b])
            .max()
            .unwrap_or(0);

        Ok(Self {
            coupling_map,
            num_qubits: max_qubit + 1,
            bidirectional,
            name: None,
        })
    }

    /// Create linear chain topology
    /// Gantree: linear(n) -> Self // 선형 체인
    ///
    /// Connectivity: 0-1-2-3-...-N-1
    pub fn linear(n: usize) -> Self {
        let coupling_map: Vec<(QubitId, QubitId)> =
            (0..n.saturating_sub(1)).map(|i| (i, i + 1)).collect();

        Self {
            coupling_map,
            num_qubits: n,
            bidirectional: true,
            name: Some(format!("linear_{}", n)),
        }
    }

    /// Create ring topology
    /// Gantree: ring(n) -> Self // 원형
    ///
    /// Connectivity: 0-1-2-...-N-1-0
    pub fn ring(n: usize) -> Self {
        let mut coupling_map: Vec<(QubitId, QubitId)> =
            (0..n.saturating_sub(1)).map(|i| (i, i + 1)).collect();

        if n > 1 {
            coupling_map.push((n - 1, 0));
        }

        Self {
            coupling_map,
            num_qubits: n,
            bidirectional: true,
            name: Some(format!("ring_{}", n)),
        }
    }

    /// Create grid topology
    /// Gantree: grid(rows, cols) -> Self // 그리드
    ///
    /// Qubit indexing: row * cols + col
    pub fn grid(rows: usize, cols: usize) -> Self {
        let mut coupling_map = Vec::new();

        for r in 0..rows {
            for c in 0..cols {
                let q = r * cols + c;

                // Right neighbor
                if c + 1 < cols {
                    coupling_map.push((q, q + 1));
                }

                // Down neighbor
                if r + 1 < rows {
                    coupling_map.push((q, q + cols));
                }
            }
        }

        Self {
            coupling_map,
            num_qubits: rows * cols,
            bidirectional: true,
            name: Some(format!("grid_{}x{}", rows, cols)),
        }
    }

    /// Create heavy-hex topology (IBM Falcon/Hummingbird style)
    pub fn heavy_hex(layers: usize) -> Self {
        // Simplified heavy-hex for common sizes
        // Full implementation would need IBM-specific patterns
        let (coupling_map, num_qubits) = match layers {
            1 => {
                // 7-qubit like IBM Lagos
                (vec![(0, 1), (1, 2), (1, 3), (3, 5), (4, 5), (5, 6)], 7)
            }
            2 => {
                // 27-qubit like IBM Falcon
                // Simplified pattern
                let mut map = Vec::new();
                for i in 0..26 {
                    if i % 5 != 4 {
                        map.push((i, i + 1));
                    }
                }
                (map, 27)
            }
            _ => {
                // Default to linear for unsupported sizes
                return Self::linear(7 * layers);
            }
        };

        Self {
            coupling_map,
            num_qubits,
            bidirectional: true,
            name: Some(format!("heavy_hex_{}", layers)),
        }
    }

    /// Create all-to-all topology (ideal)
    pub fn all_to_all(n: usize) -> Self {
        let mut coupling_map = Vec::new();

        for i in 0..n {
            for j in i + 1..n {
                coupling_map.push((i, j));
            }
        }

        Self {
            coupling_map,
            num_qubits: n,
            bidirectional: true,
            name: Some(format!("all_to_all_{}", n)),
        }
    }

    // ========================================================================
    // Properties
    // ========================================================================

    /// Get number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get coupling map
    pub fn coupling_map(&self) -> &[(QubitId, QubitId)] {
        &self.coupling_map
    }

    /// Check if bidirectional
    pub fn is_bidirectional(&self) -> bool {
        self.bidirectional
    }

    /// Get topology name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set topology name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Get number of edges
    pub fn num_edges(&self) -> usize {
        self.coupling_map.len()
    }

    // ========================================================================
    // Connectivity Queries
    // ========================================================================

    /// Check if two qubits are directly connected
    /// Gantree: is_connected(q1, q2) -> bool // 연결 여부
    pub fn is_connected(&self, q1: QubitId, q2: QubitId) -> bool {
        if q1 == q2 {
            return true;
        }

        let has_forward = self.coupling_map.contains(&(q1, q2));

        if self.bidirectional {
            has_forward || self.coupling_map.contains(&(q2, q1))
        } else {
            has_forward
        }
    }

    /// Get neighbors of a qubit
    /// Gantree: neighbors(q) -> Vec<QubitId> // 이웃
    pub fn neighbors(&self, qubit: QubitId) -> Vec<QubitId> {
        let mut neighbors = HashSet::new();

        for &(q1, q2) in &self.coupling_map {
            if q1 == qubit {
                neighbors.insert(q2);
            }
            if self.bidirectional && q2 == qubit {
                neighbors.insert(q1);
            }
        }

        let mut result: Vec<_> = neighbors.into_iter().collect();
        result.sort();
        result
    }

    /// Get qubit degree (number of connections)
    pub fn degree(&self, qubit: QubitId) -> usize {
        self.neighbors(qubit).len()
    }

    /// Build adjacency list representation
    fn adjacency_list(&self) -> HashMap<QubitId, Vec<QubitId>> {
        let mut adj: HashMap<QubitId, Vec<QubitId>> = HashMap::new();

        for i in 0..self.num_qubits {
            adj.insert(i, Vec::new());
        }

        for &(q1, q2) in &self.coupling_map {
            adj.entry(q1).or_default().push(q2);
            if self.bidirectional {
                adj.entry(q2).or_default().push(q1);
            }
        }

        adj
    }

    /// Find shortest path between two qubits (BFS)
    /// Gantree: shortest_path(q1, q2) -> Option<Vec> // 최단 경로
    pub fn shortest_path(&self, start: QubitId, end: QubitId) -> Option<Vec<QubitId>> {
        if start == end {
            return Some(vec![start]);
        }

        if start >= self.num_qubits || end >= self.num_qubits {
            return None;
        }

        let adj = self.adjacency_list();
        let mut visited = vec![false; self.num_qubits];
        let mut parent: Vec<Option<QubitId>> = vec![None; self.num_qubits];
        let mut queue = VecDeque::new();

        visited[start] = true;
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            if current == end {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = Some(end);

                while let Some(n) = node {
                    path.push(n);
                    node = parent[n];
                }

                path.reverse();
                return Some(path);
            }

            if let Some(neighbors) = adj.get(&current) {
                for &neighbor in neighbors {
                    if !visited[neighbor] {
                        visited[neighbor] = true;
                        parent[neighbor] = Some(current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        None
    }

    /// Calculate distance between two qubits
    pub fn distance(&self, q1: QubitId, q2: QubitId) -> Option<usize> {
        self.shortest_path(q1, q2).map(|p| p.len() - 1)
    }

    /// Check if topology is connected (all qubits reachable)
    pub fn is_fully_connected(&self) -> bool {
        if self.num_qubits <= 1 {
            return true;
        }

        for q in 1..self.num_qubits {
            if self.shortest_path(0, q).is_none() {
                return false;
            }
        }

        true
    }

    // ========================================================================
    // Validation
    // ========================================================================

    /// Validate a circuit against this topology
    /// Gantree: validate_circuit(&self, Circuit) -> Result // 검증
    pub fn validate_circuit(&self, circuit: &Circuit) -> NisoResult<()> {
        if circuit.num_qubits() > self.num_qubits {
            return Err(NisoError::QubitOutOfRange {
                qubit: circuit.num_qubits() - 1,
                max: self.num_qubits - 1,
            });
        }

        for (q1, q2) in circuit.two_qubit_pairs() {
            if !self.is_connected(q1, q2) {
                return Err(NisoError::TopologyViolation { q1, q2 });
            }
        }

        Ok(())
    }

    // ========================================================================
    // Topology Analysis
    // ========================================================================

    /// Get diameter (maximum shortest path length)
    pub fn diameter(&self) -> usize {
        let mut max_dist = 0;

        for i in 0..self.num_qubits {
            for j in i + 1..self.num_qubits {
                if let Some(d) = self.distance(i, j) {
                    max_dist = max_dist.max(d);
                }
            }
        }

        max_dist
    }

    /// Get average degree
    pub fn average_degree(&self) -> f64 {
        if self.num_qubits == 0 {
            return 0.0;
        }

        let total_degree: usize = (0..self.num_qubits).map(|q| self.degree(q)).sum();
        total_degree as f64 / self.num_qubits as f64
    }

    /// Find qubits with minimum degree (potential bottlenecks)
    pub fn min_degree_qubits(&self) -> Vec<QubitId> {
        if self.num_qubits == 0 {
            return vec![];
        }

        let degrees: Vec<usize> = (0..self.num_qubits).map(|q| self.degree(q)).collect();
        let min_degree = *degrees.iter().min().unwrap_or(&0);

        degrees
            .iter()
            .enumerate()
            .filter(|(_, &d)| d == min_degree)
            .map(|(i, _)| i)
            .collect()
    }

    /// Find linear chain of given length (for TQQC)
    pub fn find_linear_chain(&self, length: usize) -> Option<Vec<QubitId>> {
        if length > self.num_qubits {
            return None;
        }

        if length <= 1 {
            return Some(vec![0]);
        }

        // Try starting from each qubit
        for start in 0..self.num_qubits {
            if let Some(chain) = self.find_chain_from(start, length) {
                return Some(chain);
            }
        }

        None
    }

    fn find_chain_from(&self, start: QubitId, length: usize) -> Option<Vec<QubitId>> {
        let mut chain = vec![start];
        let mut visited = HashSet::new();
        visited.insert(start);

        while chain.len() < length {
            let current = *chain.last()?;
            let neighbors = self.neighbors(current);

            // Find unvisited neighbor
            let next = neighbors.into_iter().find(|n| !visited.contains(n))?;

            chain.push(next);
            visited.insert(next);
        }

        Some(chain)
    }
}

// ============================================================================
// Display
// ============================================================================

impl std::fmt::Display for Topology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Topology({} qubits, {} edges{})",
            self.num_qubits,
            self.num_edges(),
            self.name
                .as_ref()
                .map(|n| format!(", {}", n))
                .unwrap_or_default()
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::CircuitBuilder;

    #[test]
    fn test_linear_topology() {
        let topo = Topology::linear(5);

        assert_eq!(topo.num_qubits(), 5);
        assert_eq!(topo.num_edges(), 4);
        assert!(topo.is_connected(0, 1));
        assert!(topo.is_connected(1, 2));
        assert!(!topo.is_connected(0, 2)); // Not directly connected
    }

    #[test]
    fn test_ring_topology() {
        let topo = Topology::ring(4);

        assert_eq!(topo.num_qubits(), 4);
        assert_eq!(topo.num_edges(), 4);
        assert!(topo.is_connected(3, 0)); // Ring closure
    }

    #[test]
    fn test_grid_topology() {
        let topo = Topology::grid(2, 3);

        assert_eq!(topo.num_qubits(), 6);
        // 2 horizontal edges per row × 2 rows = 4
        // 3 vertical edges per col × 1 = 3
        // Total = 7
        assert_eq!(topo.num_edges(), 7);
    }

    #[test]
    fn test_neighbors() {
        let topo = Topology::linear(5);

        assert_eq!(topo.neighbors(0), vec![1]);
        assert_eq!(topo.neighbors(2), vec![1, 3]);
        assert_eq!(topo.neighbors(4), vec![3]);
    }

    #[test]
    fn test_shortest_path() {
        let topo = Topology::linear(5);

        let path = topo.shortest_path(0, 4).unwrap();
        assert_eq!(path, vec![0, 1, 2, 3, 4]);

        let path = topo.shortest_path(0, 0).unwrap();
        assert_eq!(path, vec![0]);
    }

    #[test]
    fn test_validate_circuit_valid() {
        let topo = Topology::linear(5);
        let circuit = CircuitBuilder::new(5).h(0).cx_chain().build();

        assert!(topo.validate_circuit(&circuit).is_ok());
    }

    #[test]
    fn test_validate_circuit_invalid() {
        let topo = Topology::linear(5);
        let mut circuit = CircuitBuilder::new(5).build();

        // Try to add non-adjacent CNOT
        circuit.add_gate(crate::gate::Gate::Cnot(0, 3)).unwrap();

        assert!(topo.validate_circuit(&circuit).is_err());
    }

    #[test]
    fn test_find_linear_chain() {
        let topo = Topology::linear(7);

        let chain = topo.find_linear_chain(5).unwrap();
        assert_eq!(chain.len(), 5);

        // Verify chain is valid
        for i in 0..chain.len() - 1 {
            assert!(topo.is_connected(chain[i], chain[i + 1]));
        }
    }

    #[test]
    fn test_diameter() {
        let linear = Topology::linear(5);
        assert_eq!(linear.diameter(), 4);

        let ring = Topology::ring(6);
        assert_eq!(ring.diameter(), 3); // Max distance in ring of 6
    }

    #[test]
    fn test_all_to_all() {
        let topo = Topology::all_to_all(4);

        // All pairs should be connected
        for i in 0..4 {
            for j in 0..4 {
                assert!(topo.is_connected(i, j));
            }
        }
    }
}
