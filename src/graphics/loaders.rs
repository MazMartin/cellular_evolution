use super::models::cpu::Primitive;
use super::models::gpu::{GpuPrimitive, GpuPrimitiveIndex, GpuQuadRenderInstance};
use super::models::space::AABB;
use crate::core::sim::AppContext;
use crate::utils::algorithms;
use crate::utils::data::IdxPair;
use std::sync::{Arc, Mutex};

pub struct EnvironmentRenderLoader {
    flatten_lookup: Vec<usize>,
    primitives: Vec<Primitive>,
    connections: Vec<IdxPair>,

    pub gpu_primitives: Vec<GpuPrimitive>,
    pub gpu_primitive_indices: Vec<GpuPrimitiveIndex>,
    pub gpu_render_instances: Vec<GpuQuadRenderInstance>,
}

impl EnvironmentRenderLoader {
    pub(crate) fn new() -> Self {
        Self {
            flatten_lookup: vec![0; 100],
            primitives: Vec::with_capacity(100),
            connections: Vec::with_capacity(100),

            gpu_primitives: Vec::with_capacity(100),
            gpu_primitive_indices: Vec::with_capacity(100),
            gpu_render_instances: Vec::with_capacity(100),
        }
    }

    fn flush(&mut self) {
        self.flatten_lookup = vec![0; 100];
        self.primitives.clear();
        self.connections.clear();

        self.gpu_primitives.clear();
        self.gpu_primitive_indices.clear();
        self.gpu_render_instances.clear();
    }

    pub fn run(&mut self, state: Arc<Mutex<AppContext>>) {
        self.flush();
        {
            let mut state = state.lock().expect("Failed to lock SimulationState");
            self.access(&mut state);
        }
        self.process();
    }

    fn access(&mut self, state: &mut AppContext) {
        // flatten whole cell array
        for (og_index, flat_index, cell) in state.cells.flatten_enumerate() {
            // append to lookup
            self.flatten_lookup[og_index] = flat_index;

            // handle the primitives
            let mut cell_primitives = cell.typ.get_membrane_primitive();
            cell_primitives.transform = cell.get_transform() * cell_primitives.transform;
            self.primitives.push(cell_primitives);
        }

        for connection in state.connections.iter() {
            self.connections
                .push(IdxPair::new(connection.id_a, connection.id_b));
        }
    }

    fn process(&mut self) {
        self.connections.iter_mut().for_each(|c| {
            c.a = self.flatten_lookup[c.a];
            c.b = self.flatten_lookup[c.b];
        });

        let group_csr =
            algorithms::CSR::groups_from_connections(&self.connections, self.primitives.len() - 1);
        let primitive_indices = group_csr.indices;
        let render_instances = group_csr.indptr;

        self.gpu_render_instances = render_instances
            .iter()
            .map(|instance| {
                let Some((&first_index, rest_indices)) =
                    primitive_indices[instance.a..instance.b].split_first()
                else {
                    panic!("Primitive slice is empty");
                };

                let mut aabb_union = AABB::UNIT.transformed(self.primitives[first_index].transform);

                for &index in rest_indices {
                    let sub_transform = self.primitives[index].transform;
                    let sub_aabb = AABB::UNIT.transformed(sub_transform);
                    aabb_union = aabb_union.union(&sub_aabb);
                }

                GpuQuadRenderInstance {
                    aabb_center: aabb_union.center.to_array(),
                    aabb_half: aabb_union.half.to_array(),
                    start_i: instance.a as u32,
                    end_i: instance.b as u32,
                }
            })
            .collect();

        self.gpu_primitive_indices = primitive_indices
            .iter()
            .cloned()
            .map(GpuPrimitiveIndex::from)
            .collect();
        self.gpu_primitives = self
            .primitives
            .iter()
            .cloned()
            .map(GpuPrimitive::from)
            .collect();
    }
}
