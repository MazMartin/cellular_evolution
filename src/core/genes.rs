use super::features::CellType;

struct GeneticCode {}
pub struct Gene {
    pub stems: Vec<Self>,
    pub typ: CellType,
}

impl Gene {
    pub fn leaf_node(typ: CellType) -> Self {
        Self { stems: vec![], typ }
    }
}
