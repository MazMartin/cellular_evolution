use crate::core::elements::CellConnection;
use crate::core::sim::{SimContext, SimulationState};
use crate::core::{elements::Cell, features::CellType, genes::Gene};
use crate::graphics::models::space::AABB;
use glam::Vec2;
use rand::prelude::*;
use std::f64::consts::TAU;

/// Creates a sample organism with cells arranged at corners of a bounding box and connected to a central neural cell.
pub fn organism_lookn_cells(context: SimContext) -> SimulationState {
    let bound = AABB::UNIT * 4.0;

    let mut cell_alloc = SimulationState::new(context);

    // Insert cells at center and corners with different cell types
    cell_alloc.cells.insert_alloc_vec(vec![
        Cell::new(Vec2::new(0.0, 0.0).into(), CellType::Neural),
        Cell::new(bound.corners().bl.into(), CellType::Spore),
        Cell::new(bound.corners().br.into(), CellType::Intestinal),
        Cell::new(bound.corners().tl.into(), CellType::Muscle),
        Cell::new(bound.corners().tr.into(), CellType::Kidney),
    ]);

    let q = TAU / 4.0;

    // Connect the central neural cell to each corner cell
    cell_alloc.connections.push(CellConnection::new(0, 0. * q, 1, 0.0));
    cell_alloc.connections.push(CellConnection::new(0, 1. * q, 2, 0.0));
    cell_alloc.connections.push(CellConnection::new(0, 2. * q, 3, 0.0));
    cell_alloc.connections.push(CellConnection::new(0, 3. * q, 4, 0.0));

    cell_alloc
}

/// Creates a gene structure representing a neural cell with four leaf nodes of various cell types.
pub fn organism_lookn_gene() -> Gene {
    Gene {
        stems: vec![
            Gene::leaf_node(CellType::Kidney),
            Gene::leaf_node(CellType::Spore),
            Gene::leaf_node(CellType::Muscle),
            Gene::leaf_node(CellType::Kidney),
        ],
        typ: CellType::Neural,
    }
}

/// Returns a random position within given bounds using the provided random number generator.
pub fn random_pos_in_bounds(rng: &mut impl Rng, bound: AABB) -> Vec2 {
    let (min, max) = (bound.min(), bound.max());
    let x = rng.gen_range(min.x..=max.x);
    let y = rng.gen_range(min.y..=max.y);
    Vec2::new(x, y)
}


/// Creates a sample organism consisting of a single neural cell centered in the bounds.
pub fn organism_single_cell(context: SimContext) -> SimulationState {
    let bound = AABB::UNIT * 4.0;
    let center = bound.center;

    let mut state = SimulationState::new(context);

    // Insert one cell in the center
    state.cells.insert_alloc_vec(vec![
        Cell::new(center.into(), CellType::Fat)
    ]);

    state
}