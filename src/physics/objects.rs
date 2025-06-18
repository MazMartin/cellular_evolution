use std::f64::consts::PI;

/// Trait for 2D objects that provide mass and rotational inertia.
pub trait ObjectData2D {
    fn mass(&self) -> f64;
    fn rotational_inertia(&self) -> f64;
}

/// Represents a solid disk with radius and density.
pub struct Disk {
    pub radius: f64,
    pub density: f64,
}

impl Default for Disk {
    /// Creates a default disk with radius and density of 1.
    fn default() -> Self {
        Self {
            radius: 1.0,
            density: 1.0,
        }
    }
}

impl Disk {
    /// Creates a disk from given radius and density.
    pub fn new(radius: f64, density: f64) -> Self {
        Self { radius, density }
    }

    /// Creates a disk from mass and radius, computing density automatically.
    pub fn from_mass(mass: f64, radius: f64) -> Self {
        let area = PI * radius * radius;
        let density = if area != 0.0 { mass / area } else { 0.0 };
        Self::new(radius, density)
    }
}

impl ObjectData2D for Disk {
    /// Calculates the disk's mass using area and density.
    fn mass(&self) -> f64 {
        let area = PI * self.radius * self.radius;
        area * self.density
    }

    /// Calculates rotational inertia of the disk (solid cylinder formula).
    fn rotational_inertia(&self) -> f64 {
        0.5 * self.radius * self.radius * self.mass()
    }
}
