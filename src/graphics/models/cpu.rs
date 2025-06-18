use super::space::SrtTransform;

/// Offset used for distinguishing star-shaped polygons (e.g. pentagram vs pentagon).
const STAR_OFFSET: u32 = 10;

/// Enum representing various polygonal shapes and their star-shaped variants.
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum ShapeDesc {
    Circle = 0,
    Triangle = 3,
    Square = 4,
    Pentagon = 5,
    Pentagram = 5 + STAR_OFFSET,
    Hexagon = 6,
    Hexagram = 6 + STAR_OFFSET,
    Heptagon = 7,
    Heptagram = 7 + STAR_OFFSET,
    Octagon = 8,
    Octagram = 8 + STAR_OFFSET,
    Nonagon = 9,
    Enneagram = 9 + STAR_OFFSET,
    Decagon = 10,
    Decagram = 10 + STAR_OFFSET,
}

/// RGBA color representation.
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0, a: 255 };
    pub const BROWN: Color = Color { r: 139, g: 69, b: 19, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const PURPLE: Color = Color { r: 128, g: 0, b: 128, a: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const GRAY: Color = Color { r: 128, g: 128, b: 128, a: 255 };
}

/// A drawable primitive shape with color and transformation.
#[derive(Clone, Copy, Debug)]
pub struct Primitive {
    pub(crate) shape: ShapeDesc,
    pub(crate) color: Color,
    pub(crate) transform: SrtTransform,
}

impl Default for Primitive {
    fn default() -> Self {
        Self {
            shape: ShapeDesc::Circle,
            color: Color::PURPLE,
            transform: SrtTransform::default(),
        }
    }
}
