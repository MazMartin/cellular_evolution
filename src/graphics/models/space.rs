use crate::graphics::models::gpu::GpuVertex;
use glam::{Mat4, Vec2};
use std::ops::{BitAnd, BitOr, Div, Mul};

/// Represents a 2D Scale-Rotate-Translate transform.
///
/// Stores translation (`Vec2`), rotation in radians (`f32`), and scale (`Vec2`).
/// This transform can be converted to a 4x4 matrix for GPU use.
#[derive(Clone, Copy, Debug)]
pub struct SrtTransform {
    /// Translation vector
    pub translate: Vec2,
    /// Rotation angle in radians
    pub rotate: f32,
    /// Scale vector
    pub scale: Vec2,
}

impl Default for SrtTransform {
    /// Creates the identity transform (no translation, zero rotation, scale = 1)
    fn default() -> Self {
        Self {
            translate: Vec2::ZERO,
            rotate: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Mul for SrtTransform {
    type Output = Self;

    /// Component-wise multiply of two transforms:
    /// Translations and rotations are added,
    /// scales are multiplied component-wise.
    ///
    /// Note: This does not apply rotation of the left operand
    /// to the translation of the right operand.
    fn mul(self, rhs: Self) -> Self {
        Self {
            translate: self.translate + rhs.translate,
            rotate: self.rotate + rhs.rotate,
            scale: self.scale * rhs.scale,
        }
    }
}

impl SrtTransform {
    /// Converts the SRT transform to a 4x4 matrix suitable for GPU shaders.
    ///
    /// The order is translation * rotation * scale.
    pub fn to_mat4(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.translate.extend(0.0));
        let rotation = Mat4::from_rotation_z(self.rotate);
        let scale = Mat4::from_scale(self.scale.extend(1.0));
        translation * rotation * scale
    }
}

/// Axis-Aligned Bounding Box (AABB) in 2D.
///
/// Defined by center and half-extents along X and Y axes.
/// Used for spatial queries, culling, and bounding volume calculations.
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    /// Center point of the bounding box
    pub center: Vec2,
    /// Half the width and height of the bounding box
    pub half: Vec2,
}

impl AABB {
    /// Unit AABB at origin with half-extents (1, 1)
    pub const UNIT: Self = Self {
        center: Vec2::ZERO,
        half: Vec2::ONE,
    };

    /// Creates a new AABB from given center and half-extents
    pub fn new(center: Vec2, half: Vec2) -> Self {
        Self { center, half }
    }

    /// Creates an AABB centered at origin with width and height given by `wh`
    pub fn from_wh(wh: Vec2) -> Self {
        Self {
            center: Vec2::ZERO,
            half: wh / 2.0,
        }
    }

    /// Creates an AABB from minimum and maximum corner points
    pub fn from_edges(min: Vec2, max: Vec2) -> Self {
        let center = (min + max) * 0.5;
        let half = (max - min) * 0.5;
        Self { center, half }
    }

    /// Returns the width and height of the bounding box
    pub fn wh(&self) -> Vec2 {
        self.half * 2.0
    }

    /// Returns the minimum corner (bottom-left)
    pub fn min(&self) -> Vec2 {
        self.center - self.half
    }

    /// Returns the maximum corner (top-right)
    pub fn max(&self) -> Vec2 {
        self.center + self.half
    }

    /// Returns the width of the bounding box
    pub fn width(&self) -> f32 {
        self.half.x * 2.0
    }

    /// Returns the height of the bounding box
    pub fn height(&self) -> f32 {
        self.half.y * 2.0
    }

    /// Returns the left-center edge point
    pub fn left(&self) -> Vec2 {
        Vec2::new(self.center.x - self.half.x, self.center.y)
    }

    /// Returns the right-center edge point
    pub fn right(&self) -> Vec2 {
        Vec2::new(self.center.x + self.half.x, self.center.y)
    }

    /// Returns the top-center edge point
    pub fn top(&self) -> Vec2 {
        Vec2::new(self.center.x, self.center.y + self.half.y)
    }

    /// Returns the bottom-center edge point
    pub fn bottom(&self) -> Vec2 {
        Vec2::new(self.center.x, self.center.y - self.half.y)
    }

    /// Converts the bounding box to a forward projection transform
    /// with translation = center and scale = half-extents.
    pub fn to_forward_projection(&self) -> SrtTransform {
        SrtTransform {
            translate: self.center,
            scale: self.half,
            ..Default::default()
        }
    }

    /// Applies a translation and scale transform to the bounding box.
    /// Rotation is not applied to keep axis-alignment.
    pub fn transform(&mut self, transform: SrtTransform) {
        self.center += transform.translate;
        self.half *= transform.scale;
    }

    /// Returns a transformed copy of this bounding box by given transform.
    pub fn transformed(self, transform: SrtTransform) -> Self {
        let mut copy = self;
        copy.transform(transform);
        copy
    }

    /// Returns the four corners of the bounding box as a `QuadVerts`
    /// with top-left, top-right, bottom-left, bottom-right corners.
    pub fn corners(&self) -> QuadVerts {
        let (min, max) = (self.min(), self.max());

        QuadVerts {
            tl: Vec2::new(min.x, max.y),
            tr: Vec2::new(max.x, max.y),
            bl: Vec2::new(min.x, min.y),
            br: Vec2::new(max.x, min.y),
        }
    }

    /// Returns the union of this AABB and another,
    /// i.e. the smallest AABB containing both.
    pub fn union(&self, other: &AABB) -> AABB {
        let (min_a, max_a) = (self.min(), self.max());
        let (min_b, max_b) = (other.min(), other.max());
        let min = min_a.min(min_b);
        let max = max_a.max(max_b);
        AABB::from_edges(min, max)
    }

    /// Returns a new AABB resized proportionally to fit the given aspect ratio,
    /// expanding either width or height to match the ratio without shrinking.
    pub fn max_proportional(&self, aspect: f32) -> AABB {
        let dim = self.wh();
        let max_width_for_height = dim.y * aspect;
        let max_height_for_width = dim.x / aspect;

        AABB {
            center: self.center,
            half: if max_width_for_height <= dim.x {
                Vec2::new(max_width_for_height / 2.0, self.half.y)
            } else {
                Vec2::new(self.half.x, max_height_for_width / 2.0)
            },
        }
    }

    /// Returns a new AABB padded by `padding` units on all sides.
    pub fn add_padding(&self, padding: f32) -> AABB {
        AABB {
            center: self.center,
            half: self.half + Vec2::new(padding, padding),
        }
    }
}

impl Mul<f32> for AABB {
    type Output = AABB;

    /// Scales the bounding box's half-extents by the scalar value.
    fn mul(self, scale: f32) -> AABB {
        AABB {
            center: self.center,
            half: self.half * scale,
        }
    }
}

impl Div<f32> for AABB {
    type Output = AABB;

    /// Divides the bounding box's center and half-extents by the scalar value.
    fn div(self, rhs: f32) -> AABB {
        AABB {
            center: self.center / rhs,
            half: self.half / rhs,
        }
    }
}

impl Div<AABB> for AABB {
    type Output = AABB;

    /// Computes a relative transform between two AABBs.
    ///
    /// The center is offset by rhs.center, then divided by rhs half-extents,
    /// and half-extents scaled accordingly.
    fn div(self, rhs: AABB) -> AABB {
        AABB {
            center: (self.center - rhs.center) / rhs.half,
            half: self.half / rhs.half,
        }
    }
}

impl BitOr for AABB {
    type Output = AABB;

    /// Returns the union of two AABBs (smallest bounding box containing both)
    fn bitor(self, rhs: AABB) -> AABB {
        let min = self.min().min(rhs.min());
        let max = self.max().max(rhs.max());
        AABB::from_edges(min, max)
    }
}

impl BitAnd for AABB {
    type Output = AABB;

    /// Returns the intersection of two AABBs (overlapping region).
    /// If no overlap exists, returns an empty box centered between min and max.
    fn bitand(self, rhs: AABB) -> AABB {
        let min = self.min().max(rhs.min());
        let max = self.max().min(rhs.max());

        if min.x > max.x || min.y > max.y {
            // No overlap; return empty AABB
            return AABB::new((min + max) * 0.5, Vec2::ZERO);
        }

        AABB::from_edges(min, max)
    }
}

/// Four corner vertices of a quad, used for rendering or spatial calculations.
#[derive(Clone, Copy, Debug)]
pub struct QuadVerts {
    /// Top-left corner
    pub tl: Vec2,
    /// Top-right corner
    pub tr: Vec2,
    /// Bottom-left corner
    pub bl: Vec2,
    /// Bottom-right corner
    pub br: Vec2,
}

impl QuadVerts {
    /// Returns the max coordinates among all corners
    pub fn max(&self) -> Vec2 {
        let xs = [self.tl.x, self.tr.x, self.bl.x, self.br.x];
        let ys = [self.tl.y, self.tr.y, self.bl.y, self.br.y];
        Vec2::new(
            xs.iter().copied().fold(f32::NEG_INFINITY, f32::max),
            ys.iter().copied().fold(f32::NEG_INFINITY, f32::max),
        )
    }

    /// Returns the min coordinates among all corners
    pub fn min(&self) -> Vec2 {
        let xs = [self.tl.x, self.tr.x, self.bl.x, self.br.x];
        let ys = [self.tl.y, self.tr.y, self.bl.y, self.br.y];
        Vec2::new(
            xs.iter().copied().fold(f32::INFINITY, f32::min),
            ys.iter().copied().fold(f32::INFINITY, f32::min),
        )
    }

    /// Returns vertices ordered counter-clockwise (CCW) without repeating start vertex.
    /// Order: top-left, top-right, bottom-right, bottom-left
    pub fn ccw(&self) -> [GpuVertex; 4] {
        [
            self.tl.into(), self.tr.into(),
            self.br.into(), self.bl.into(),
        ]
    }

    /// Returns vertices ordered clockwise (CW) without repeating start vertex.
    /// Order: top-left, bottom-left, bottom-right, top-right
    pub fn cw(&self) -> [GpuVertex; 4] {
        [
            self.tl.into(), self.bl.into(),
            self.br.into(), self.tr.into(),
        ]
    }

    /// Returns CCW loop with repeated start vertex for closed line strip.
    pub fn ccw_loop(&self) -> [GpuVertex; 5] {
        [
            self.tl.into(), self.tr.into(),
            self.br.into(), self.bl.into(),
            self.tl.into(),
        ]
    }

    /// Returns CW loop with repeated start vertex for closed line strip.
    pub fn cw_loop(&self) -> [GpuVertex; 5] {
        [
            self.tl.into(), self.bl.into(),
            self.br.into(), self.tr.into(),
            self.tl.into(),
        ]
    }

    /// Returns vertices as two triangles in CCW order forming the quad mesh.
    pub fn ccw_mesh(&self) -> [GpuVertex; 6] {
        [
            self.tl.into(), self.bl.into(), self.tr.into(),
            self.tr.into(), self.bl.into(), self.br.into(),
        ]
    }

    /// Returns vertices as two triangles in CW order forming the quad mesh.
    pub fn cw_mesh(&self) -> [GpuVertex; 6] {
        [
            self.tl.into(), self.tr.into(), self.bl.into(),
            self.tr.into(), self.br.into(), self.bl.into(),
        ]
    }
}

/// Oriented Bounding Box in 2D.
///
/// Defined by center, half-extents, and rotation angle (radians).
#[derive(Clone, Copy, Debug)]
pub struct OBB {
    /// Center of the box
    pub center: Vec2,
    /// Half-width and half-height
    pub half: Vec2,
    /// Rotation angle in radians
    pub angle: f32,
}

impl OBB {
    /// Computes the four corners of the OBB as a `QuadVerts` struct.
    pub fn corners(&self) -> QuadVerts {
        let cos_a = self.angle.cos();
        let sin_a = self.angle.sin();

        let right = Vec2::new(cos_a, sin_a) * self.half.x;
        let up = Vec2::new(-sin_a, cos_a) * self.half.y;

        let tl = self.center - right + up;
        let tr = self.center + right + up;
        let bl = self.center - right - up;
        let br = self.center + right - up;

        QuadVerts { tl, tr, bl, br }
    }

    /// Fits and returns an axis-aligned bounding box that fully contains this OBB.
    pub fn fit_aabb(&self) -> AABB {
        let corners = self.corners();
        AABB::from_edges(corners.min(), corners.max())
    }
}

/// Represents a 2D camera with a rectangular viewport.
struct Camera {
    /// Viewport bounds as an AABB in world coordinates
    viewport: AABB,
}