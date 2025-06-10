use crate::graphics::models::gpu::GpuVertex;
use glam::{Mat4, Vec2};
use std::ops::{BitAnd, BitOr, Div, Mul};

#[derive(Clone, Copy, Debug)]
pub struct SrtTransform {
    pub translate: Vec2,
    pub rotate: f32,
    pub scale: Vec2,
}

impl Default for SrtTransform {
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

    fn mul(self, rhs: Self) -> Self {
        Self {
            translate: self.translate + rhs.translate,
            rotate: self.rotate + rhs.rotate,
            scale: self.scale * rhs.scale,
        }
    }
}

impl SrtTransform {
    pub fn to_mat4(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.translate.extend(0.0));
        let rotation = Mat4::from_rotation_z(self.rotate);
        let scale = Mat4::from_scale(self.scale.extend(1.0));
        translation * rotation * scale
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub center: Vec2,
    pub half: Vec2,
}

impl AABB {
    pub const UNIT: Self = Self {
        center: Vec2::ZERO,
        half: Vec2::ONE,
    };

    pub fn new(center: Vec2, half: Vec2) -> Self {
        Self { center, half }
    }

    pub fn from_wh(wh: Vec2) -> Self {
        Self {
            center: Vec2::ZERO,
            half: wh / 2.0,
        }
    }

    pub fn from_edges(min: Vec2, max: Vec2) -> Self {
        let center = (min + max) * 0.5;
        let half = (max - min) * 0.5;
        Self { center, half }
    }

    pub fn wh(&self) -> Vec2 {
        self.half * 2.0
    }
    pub fn min(&self) -> Vec2 {
        self.center - self.half
    }
    pub fn max(&self) -> Vec2 {
        self.center + self.half
    }
    pub fn width(&self) -> f32 {
        self.half.x * 2.0
    }
    pub fn height(&self) -> f32 {
        self.half.y * 2.0
    }
    pub fn left(&self) -> Vec2 {
        Vec2::new(self.center.x - self.half.x, self.center.y)
    }
    pub fn right(&self) -> Vec2 {
        Vec2::new(self.center.x + self.half.x, self.center.y)
    }
    pub fn top(&self) -> Vec2 {
        Vec2::new(self.center.x, self.center.y + self.half.y)
    }
    pub fn bottom(&self) -> Vec2 {
        Vec2::new(self.center.x, self.center.y - self.half.y)
    }

    pub fn to_forward_projection(&self) -> SrtTransform {
        SrtTransform {
            translate: self.center,
            scale: self.half,
            ..Default::default()
        }
    }

    pub fn transform(&mut self, transform: SrtTransform) {
        self.center += transform.translate;
        self.half *= transform.scale;
    }

    pub fn transformed(self, transform: SrtTransform) -> Self {
        let mut copy = self;
        copy.transform(transform);
        copy
    }

    pub fn corners(&self) -> QuadVerts {
        let (min, max) = (self.min(), self.max());

        QuadVerts {
            tl: Vec2::new(min.x, max.y),
            tr: Vec2::new(max.x, max.y),
            bl: Vec2::new(min.x, min.y),
            br: Vec2::new(max.x, min.y),
        }
    }

    pub fn union(&self, other: &AABB) -> AABB {
        let (min_a, max_a) = (self.min(), self.max());
        let (min_b, max_b) = (other.min(), other.max());
        let min = min_a.min(min_b);
        let max = max_a.max(max_b);
        AABB::from_edges(min, max)
    }

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
    
    pub fn add_padding(&self, padding: f32) -> AABB {
        AABB {
            center: self.center,
            half: self.half + Vec2::new(padding, padding),
        }
    }
}

impl Mul<f32> for AABB {
    type Output = AABB;
    fn mul(self, scale: f32) -> AABB {
        AABB {
            center: self.center,
            half: self.half * scale,
        }
    }
}

impl Div<f32> for AABB {
    type Output = AABB;
    fn div(self, rhs: f32) -> AABB {
        AABB {
            center: self.center / rhs,
            half: self.half / rhs,
        }
    }
}

impl Div<AABB> for AABB {
    type Output = AABB;
    fn div(self, rhs: AABB) -> AABB {
        AABB {
            center: (self.center - rhs.center) / rhs.half,
            half: self.half / rhs.half,
        }
    }
}

impl BitOr for AABB {
    type Output = AABB;

    fn bitor(self, rhs: AABB) -> AABB {
        let min = self.min().min(rhs.min());
        let max = self.max().max(rhs.max());
        AABB::from_edges(min, max)
    }
}

impl BitAnd for AABB {
    type Output = AABB;

    fn bitand(self, rhs: AABB) -> AABB {
        let min = self.min().max(rhs.min());
        let max = self.max().min(rhs.max());

        if min.x > max.x || min.y > max.y {
            return AABB::new((min + max) * 0.5, Vec2::ZERO);
        }

        AABB::from_edges(min, max)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QuadVerts {
    pub tl: Vec2,
    pub tr: Vec2,
    pub bl: Vec2,
    pub br: Vec2,
}

impl QuadVerts {
    pub fn max(&self) -> Vec2 {
        let xs = [self.tl.x, self.tr.x, self.bl.x, self.br.x];
        let ys = [self.tl.y, self.tr.y, self.bl.y, self.br.y];
        Vec2::new(
            xs.iter().copied().fold(f32::NEG_INFINITY, f32::max),
            ys.iter().copied().fold(f32::NEG_INFINITY, f32::max),
        )
    }

    pub fn min(&self) -> Vec2 {
        let xs = [self.tl.x, self.tr.x, self.bl.x, self.br.x];
        let ys = [self.tl.y, self.tr.y, self.bl.y, self.br.y];
        Vec2::new(
            xs.iter().copied().fold(f32::INFINITY, f32::min),
            ys.iter().copied().fold(f32::INFINITY, f32::min),
        )
    }

    pub fn ccw(&self) -> [GpuVertex; 4] {
        [
            self.tl.into(), self.tr.into(),
            self.br.into(), self.bl.into(),
        ]
    }

    pub fn cw(&self) -> [GpuVertex; 4] {
        [
            self.tl.into(), self.bl.into(),
            self.br.into(), self.tr.into(),
        ]
    }

    pub fn ccw_loop(&self) -> [GpuVertex; 5] {
        [
            self.tl.into(), self.tr.into(),
            self.br.into(), self.bl.into(),
            self.tl.into(),
        ]
    }

    pub fn cw_loop(&self) -> [GpuVertex; 5] {
        [
            self.tl.into(), self.bl.into(),
            self.br.into(), self.tr.into(),
            self.tl.into(),
        ]
    }

    pub fn ccw_mesh(&self) -> [GpuVertex; 6] {
        [
            self.tl.into(), self.bl.into(), self.tr.into(),
            self.tr.into(), self.bl.into(), self.br.into(),
        ]
    }
    
    pub fn cw_mesh(&self) -> [GpuVertex; 6] {
        [
            self.tl.into(), self.tr.into(), self.bl.into(),
            self.tr.into(), self.br.into(), self.bl.into(),
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OBB {
    pub center: Vec2,
    pub half: Vec2,
    pub angle: f32,
}

impl OBB {
    fn corners(&self) -> QuadVerts {
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

    fn fit_aabb(&self) -> AABB {
        let corners = self.corners();
        AABB::from_edges(corners.min(), corners.max())
    }
}

struct Camera {
    viewport: AABB,
}
