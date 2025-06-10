use std::f64::consts::PI;

pub trait ObjectData2D {
    fn mass(&self) -> f64;
    fn rotational_inertia(&self) -> f64;
}

pub struct Disk {
    pub radius: f64,
    pub density: f64,
}

impl Default for Disk {
    fn default() -> Self {
        Self {
            radius: 1.0,
            density: 1.0,
        }
    }
}

impl Disk {
    pub fn new(radius: f64, density: f64) -> Self {
        Self { radius, density }
    }

    pub fn from_mass(mass: f64, radius: f64) -> Self {
        let area = PI * radius * radius;
        let density = if area != 0.0 { mass / area } else { 0.0 };
        Self::new(radius, density)
    }
}

impl ObjectData2D for Disk {
    fn mass(&self) -> f64 {
        let area = PI * self.radius * self.radius;
        area * self.density
    }
    fn rotational_inertia(&self) -> f64 {
        0.5 * self.radius * self.radius * self.mass()
    }
}
