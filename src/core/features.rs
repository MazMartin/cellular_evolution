use crate::graphics::models::cpu::{Color, Primitive, ShapeDesc};
use crate::graphics::models::space::SrtTransform;
use glam::Vec2;

/// Represents the biological or functional type of a cell.
/// Used for rendering and simulation classification.
#[derive(Clone, Copy, Debug)]
pub enum CellType {
    Neural,
    Muscle,
    Fat,
    Liver,
    Intestinal,
    Kidney,
    HairFollicle,
    Spore,
}

impl CellType {
    /// A static list of all possible cell types.
    pub const LIST: &'static [CellType] = &[
        CellType::Neural,
        CellType::Muscle,
        CellType::Fat,
        CellType::Liver,
        CellType::Intestinal,
        CellType::Kidney,
        CellType::HairFollicle,
        CellType::Spore,
    ];

    /// Returns the visual membrane primitive used to render this cell type.
    pub fn get_membrane_primitive(&self) -> Primitive {
        // All primitives use default transform; only shape and color vary.
        let default_transform = SrtTransform::default();

        match self {
            CellType::Neural => Primitive {
                shape: ShapeDesc::Circle,
                color: Color::BLUE,
                transform: default_transform,
            },
            CellType::Muscle => Primitive {
                shape: ShapeDesc::Hexagon,
                color: Color::RED,
                transform: default_transform,
            },
            CellType::Fat => Primitive {
                shape: ShapeDesc::Pentagon,
                color: Color::YELLOW,
                transform: default_transform,
            },
            CellType::Liver => Primitive {
                shape: ShapeDesc::Decagon,
                color: Color::BROWN,
                transform: default_transform,
            },
            CellType::Intestinal => Primitive {
                shape: ShapeDesc::Triangle,
                color: Color::GREEN,
                transform: default_transform,
            },
            CellType::Kidney => Primitive {
                shape: ShapeDesc::Heptagon,
                color: Color::PURPLE,
                transform: default_transform,
            },
            CellType::HairFollicle => Primitive {
                shape: ShapeDesc::Triangle,
                color: Color::BLACK,
                transform: default_transform,
            },
            CellType::Spore => Primitive {
                shape: ShapeDesc::Square,
                color: Color::GRAY,
                transform: default_transform,
            },
        }
    }
}
