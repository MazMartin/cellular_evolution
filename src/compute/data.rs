#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RawCell {
    position: [f32; 2],
    radius: f32,
    rotation: f32,

    group_id: u32,

    _pad: [u32; 2],
}

// impl RawCell {
//     fn new(position: [f32; 2], radius: f32, color: [f32; 4], group_id: u32, primitive: u32) -> Self {
//
//     }
// }
