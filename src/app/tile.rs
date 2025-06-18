use crate::core::sim::SimulationState;
use crate::graphics::models::space::AABB;
use crate::graphics::renderer::TileRenderer;

use glam::{vec2, Vec2};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use taffy::prelude::*;
use taffy::TaffyTree;
use wgpu::RenderPass;

/// Represents a single tile that holds multiple render layers.
pub struct Tile {
    pub render_layers: Vec<Box<dyn TileRenderer>>,
}

impl Tile {
    /// Creates a tile with no render layers.
    pub fn empty() -> Self {
        Self {
            render_layers: Vec::new(),
        }
    }
}

/// Manages layout and rendering of tiles using Taffy for layout and WGPU for drawing.
pub struct TileViewManager {
    taffy: TaffyTree,
    root: NodeId,
    tiles: HashMap<NodeId, Tile>,
    aabb_cache: HashMap<NodeId, AABB>,
}

impl TileViewManager {
    /// Constructs a new TileViewManager with a root node.
    pub fn new() -> Self {
        let mut taffy = TaffyTree::new();
        let root = taffy.new_with_children(Self::root_style(), &[]).unwrap();

        Self {
            taffy,
            root,
            tiles: HashMap::new(),
            aabb_cache: HashMap::new(),
        }
    }

    /// Returns the layout style for the root container.
    fn root_style() -> Style {
        Style {
            justify_content: Some(JustifyContent::Center),
            align_items: Some(AlignItems::Center),
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        }
    }

    /// Returns the root node ID.
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Adds a new leaf node under the given parent with the provided style.
    pub fn add_leaf(&mut self, parent: NodeId, style: Style) -> NodeId {
        let node = self.taffy.new_leaf(style).unwrap();
        self.taffy.add_child(parent, node).unwrap();
        self.tiles.insert(node, Tile::empty());
        node
    }

    /// Sets a new style for a given node.
    pub fn set_style(&mut self, node: NodeId, style: Style) {
        if let Err(e) = self.taffy.set_style(node, style) {
            eprintln!("Failed to set style for node {:?}: {:?}", node, e);
        }
    }

    /// Returns the layout size of a node as a `Vec2`.
    pub fn get_size(&self, node: NodeId) -> Vec2 {
        self.taffy.layout(node)
            .map(|layout| vec2(layout.size.width, layout.size.height))
            .unwrap_or(Vec2::ZERO)
    }

    /// Computes and returns the axis-aligned bounding box of a node.
    pub fn get_aabb(&self, node: NodeId) -> AABB {
        let layout = self.taffy.layout(node).unwrap();
        let size = vec2(layout.size.width, layout.size.height);
        let position = vec2(layout.location.x, layout.location.y);
        AABB::from_edges(position, position + size)
    }

    /// Returns the clipped AABB of a node, intersected with the root node's bounds.
    pub fn get_aabb_clipped(&self, node: NodeId) -> AABB {
        self.get_aabb(node) & self.get_aabb(self.root)
    }

    /// Adds a renderer layer to the specified node and initializes it.
    pub fn add_renderer<R: TileRenderer + 'static>(
        &mut self,
        node: NodeId,
        layer: R,
        queue: &wgpu::Queue,
    ) {
        layer.init(queue);
        if let Some(tile) = self.tiles.get_mut(&node) {
            tile.render_layers.push(Box::new(layer));
        }
    }

    /// Recomputes layout and AABB cache for all tiles based on the available window size.
    pub fn resize(&mut self, available: Vec2) {
        self.taffy.set_style(self.root, Self::root_style()).unwrap();

        let size = Size {
            width: AvailableSpace::Definite(available.x),
            height: AvailableSpace::Definite(available.y),
        };

        if let Err(e) = self.taffy.compute_layout(self.root, size) {
            eprintln!("Failed to compute layout: {:?}", e);
        }

        self.aabb_cache.clear();
        let root_bounds = self.get_aabb(self.root);

        for node in self.tiles.keys() {
            let node_bounds = self.get_aabb(*node);
            let clipped = node_bounds & root_bounds;
            self.aabb_cache.insert(*node, clipped);
        }
    }

    /// Updates all tiles with simulation state and resizes layers.
    pub fn load_all(&mut self, sim_state: Arc<Mutex<SimulationState>>, queue: &wgpu::Queue) {
        for (node_id, tile) in &mut self.tiles {
            if let Some(aabb) = self.aabb_cache.get(node_id) {
                for layer in tile.render_layers.iter_mut() {
                    layer.resize(aabb.wh(), queue);
                    layer.update_render_data(Arc::clone(&sim_state), queue);
                }
            }
        }
    }

    /// Renders all tiles using the current AABB layout and render layers.
    pub fn render_all<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        for (node_id, tile) in &self.tiles {
            if let Some(aabb) = self.aabb_cache.get(node_id) {
                let size = aabb.wh();
                if size.x <= 0.0 || size.y <= 0.0 {
                    continue; // Skip invisible tiles
                }

                render_pass.set_viewport(
                    aabb.min().x,
                    aabb.min().y,
                    size.x,
                    size.y,
                    0.0,
                    1.0,
                );

                for layer in tile.render_layers.iter() {
                    layer.render_pipeline(render_pass);
                }
            }
        }
    }

    // Future: pub fn dispatch_event(...) {}
}
