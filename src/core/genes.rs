use super::features::CellType;

/// Placeholder for a full genetic code structure.
struct GeneticCode {}

/// Represents a single gene, which may branch into other genes (stems).
/// Conceptually forms a tree structure, where leaves represent terminal cell types.
pub struct Gene {
    pub stems: Vec<Gene>,
    pub typ: CellType,
}

impl Gene {
    /// Creates a leaf node (a gene with no children) of a specific cell type.
    pub fn leaf_node(typ: CellType) -> Self {
        Self {
            stems: Vec::new(),
            typ,
        }
    }
}