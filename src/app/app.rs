use super::tile::TileViewManager;
use crate::gpu;
use hecs::World;

struct App {
    ecs: World,

}

impl App {
    fn new() -> Self {
        Self {
            ecs: World::new(),
        }
    }
}