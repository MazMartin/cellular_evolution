use super::data::IdxPair;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct CSR {
    pub indices: Vec<usize>,  // Flattened adjacency lists including self
    pub indptr: Vec<IdxPair>, // Ranges in indices per node
}

impl CSR {
    /// Builds adjacency lists (including self) from connections
    pub fn adjacent_from_connections(connections: &[IdxPair], max_index: usize) -> Self {
        let node_count = max_index + 1;

        // Degrees start at 1 to account for self-reference
        let mut degrees = vec![1usize; node_count];
        for conn in connections {
            degrees[conn.a] += 1;
            degrees[conn.b] += 1;
        }

        // Calculate offsets (indptr) based on degrees
        let mut indptr = Vec::with_capacity(node_count);
        let mut offset = 0;
        for &deg in &degrees {
            indptr.push(IdxPair::new(offset, offset + deg));
            offset += deg;
        }

        // Initialize indices vector and write positions
        let mut indices = vec![0usize; offset];
        let mut write_pos: Vec<usize> = indptr.iter().map(|p| p.a).collect();

        // Write each node's self-index first
        for node in 0..node_count {
            indices[write_pos[node]] = node;
            write_pos[node] += 1;
        }

        // Write neighbor indices from connections
        for conn in connections {
            let a = conn.a;
            let b = conn.b;
            indices[write_pos[a]] = b;
            write_pos[a] += 1;
            indices[write_pos[b]] = a;
            write_pos[b] += 1;
        }

        Self { indices, indptr }
    }

    /// Computes connected groups using BFS over adjacency lists
    pub fn groups_from_connections(connections: &[IdxPair], max_index: usize) -> Self {
        let adj = CSR::adjacent_from_connections(connections, max_index);
        let mut visited = vec![false; max_index + 1];
        let mut indices = Vec::new();
        let mut indptr = Vec::new();

        for start_node in 0..=max_index {
            if visited[start_node] {
                continue;
            }

            let mut queue = VecDeque::new();
            queue.push_back(start_node);
            visited[start_node] = true;

            let group_start_idx = indices.len();

            while let Some(node) = queue.pop_front() {
                indices.push(node);
                let IdxPair { a: start, b: end } = adj.indptr[node];
                for &neighbor in &adj.indices[start..end] {
                    if !visited[neighbor] {
                        visited[neighbor] = true;
                        queue.push_back(neighbor);
                    }
                }
            }

            let group_end_idx = indices.len();
            indptr.push(IdxPair::new(group_start_idx, group_end_idx));
        }

        CSR { indices, indptr }
    }

    /// Prints adjacency info for debugging
    pub fn print_debug(&self) {
        for (node, range) in self.indptr.iter().enumerate() {
            if range.a > range.b || range.b > self.indices.len() {
                println!("Node {}: INVALID RANGE [{}..{}]", node, range.a, range.b);
                continue;
            }
            let neighbors = &self.indices[range.a..range.b];
            println!("Node {}: {:?}", node, neighbors);
        }
    }
}

/// Iterator over adjacency rows in CSR
pub struct CSRRowIter<'a> {
    csr: &'a CSR,
    row: usize,
}

impl<'a> Iterator for CSRRowIter<'a> {
    type Item = &'a [usize];

    fn next(&mut self) -> Option<Self::Item> {
        if self.row >= self.csr.indptr.len() {
            return None;
        }
        let slice = self.csr.row(self.row);
        self.row += 1;
        Some(slice)
    }
}

impl CSR {
    /// Returns iterator over adjacency rows
    pub fn rows(&self) -> CSRRowIter<'_> {
        CSRRowIter { csr: self, row: 0 }
    }

    /// Returns the adjacency slice for node `i`
    pub fn row(&self, i: usize) -> &[usize] {
        let IdxPair { a, b } = self.indptr[i];
        &self.indices[a..b]
    }
}
