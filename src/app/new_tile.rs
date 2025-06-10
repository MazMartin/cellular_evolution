use crate::graphics::models::space::AABB;
use crate::graphics::renderer::TileRenderer;
use glam::{Vec2, vec2};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use taffy::TaffyTree;
use taffy::prelude::*;
use wgpu::RenderPass;
use crate::core::sim::SimulationState;

pub struct Tile {
    pub render_layers: Vec<Box<dyn TileRenderer>>,
}

impl Tile {
    pub fn empty() -> Self {
        Self {
            render_layers: Vec::new(),
        }
    }
}

pub struct TileViewManager {
    taffy: TaffyTree,
    root: NodeId,
    tiles: HashMap<NodeId, Tile>,
    aabb_cache: HashMap<NodeId, AABB>,
}


impl TileViewManager {
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

    // Layout Methods
    fn root_style() -> Style {
        Style {
            justify_content: Option::from(JustifyContent::Center),
            align_items: Option::from(AlignItems::Center),
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn add_leaf(&mut self, parent: NodeId, style: Style) -> NodeId {
        let node = self.taffy.new_leaf(style).unwrap();
        self.taffy.add_child(parent, node).unwrap();
        self.tiles.insert(node, Tile::empty());
        node
    }

    pub fn set_style(&mut self, node: NodeId, style: Style) {
        if let Err(e) = self.taffy.set_style(node, style) {
            eprintln!("Failed to set style for node {:?}: {:?}", node, e);
        }
    }

    pub fn get_size(&self, node: NodeId) -> Vec2 {
        if let Ok(layout) = self.taffy.layout(node) {
            Vec2::new(layout.size.width, layout.size.height)
        } else {
            Vec2::ZERO
        }
    }

    pub fn get_aabb(&self, node: NodeId) -> AABB {
        let layout = self.taffy.layout(node).unwrap();
        let size = vec2(layout.size.width, layout.size.height);
        let pos = vec2(layout.location.x, layout.location.y);
        AABB::from_edges(pos, pos + size)
    }

    pub fn get_aabb_clipped(&self, node: NodeId) -> AABB {
        self.get_aabb(node) & self.get_aabb(self.root)
    }

    // Layer Methods
    pub fn add_renderer<R: TileRenderer + 'static>(&mut self, node: NodeId, layer: R) {
        if let Some(tile) = self.tiles.get_mut(&node) {
            tile.render_layers.push(Box::new(layer));
        }
    }

    pub fn resize(&mut self, available: Vec2) {
        self.taffy
            .set_style(self.root, Self::root_style())
            .unwrap();
        let size = Size {
            width: AvailableSpace::Definite(available.x),
            height: AvailableSpace::Definite(available.y),
        };
        if let Err(e) = self.taffy.compute_layout(self.root, size) {
            eprintln!("Failed to compute layout: {:?}", e);
        }

        self.aabb_cache.clear();
        let root_aabb = self.get_aabb(self.root);

        for node in self.tiles.keys() {
            let node_aabb = self.get_aabb(*node);
            let clipped = node_aabb & root_aabb;
            self.aabb_cache.insert(*node, clipped);
        }
    }

    pub fn load_all(&mut self, context: Arc<Mutex<SimulationState>>, queue: &wgpu::Queue) {
        for (node_id, tile) in &mut self.tiles {
            if let Some(aabb) = self.aabb_cache.get(node_id) {
                for layer in tile.render_layers.iter_mut() {
                    layer.resize(aabb.wh(), queue);
                    layer.update_render_data(Arc::clone(&context), queue);
                }
            }
        }
    }

    pub fn render_all<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        for (node, layers) in &self.tiles {
            if let Some(aabb) = self.aabb_cache.get(node) {
                let size = aabb.wh();
                if size.x <= 0.0 || size.y <= 0.0 {
                    continue;
                }
                render_pass.set_viewport(aabb.min().x, aabb.min().y, size.x, size.y, 0.0, 1.0);
                for view in layers.render_layers.iter() {
                    view.render_pipeline(render_pass);
                }
            }
        }
    }

    //pub fn dispatch_event
}