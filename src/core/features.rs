use crate::graphics::models::cpu::{Color, Primitive, ShapeDesc};
use crate::graphics::models::space::SrtTransform;
use glam::Vec2;

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

    pub fn get_membrane_primitive(&self) -> Primitive {
        match self {
            CellType::Neural => Primitive {
                shape: ShapeDesc::Circle,
                color: Color::BLUE,
                transform: SrtTransform {
                    ..Default::default()
                },
            },
            CellType::Muscle => Primitive {
                shape: ShapeDesc::Hexagon,
                color: Color::RED,
                transform: SrtTransform {
                    ..Default::default()
                },
            },
            CellType::Fat => Primitive {
                shape: ShapeDesc::Pentagon,
                color: Color::YELLOW,
                transform: SrtTransform {
                    ..Default::default()
                },
            },
            CellType::Liver => Primitive {
                shape: ShapeDesc::Decagon,
                color: Color::BROWN,
                transform: SrtTransform {
                    ..Default::default()
                },
            },
            CellType::Intestinal => Primitive {
                shape: ShapeDesc::Triangle,
                color: Color::GREEN,
                transform: SrtTransform {
                    ..Default::default()
                },
            },
            CellType::Kidney => Primitive {
                shape: ShapeDesc::Heptagon,
                color: Color::PURPLE,
                transform: SrtTransform {
                    ..Default::default()
                },
            },
            CellType::HairFollicle => Primitive {
                shape: ShapeDesc::Triangle,
                color: Color::BLACK,
                transform: SrtTransform {
                    ..Default::default()
                },
            },
            CellType::Spore => Primitive {
                shape: ShapeDesc::Square,
                color: Color::GRAY,
                transform: SrtTransform {
                    ..Default::default()
                },
            },
        }
    }
}
