use super::tile::TileViewManager;
use crate::gpu;


struct App {
    gpu_context: Option<gpu::context::GpuContext>,
    tiles: TileViewManager,
}