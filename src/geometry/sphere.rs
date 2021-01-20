extern crate cgmath;

use crate::material::Material;
use crate::common::{Entity, ColliderResult, Ray};

use cgmath::{Vector3, InnerSpace};

pub struct Sphere {
    position: Vector3<f32>,
    radius: f32,
    radius2: f32,
    material: Material,
}

impl Sphere {
    pub fn new(position: Vector3<f32>, radius: f32, material: Material) -> Sphere {
        Sphere {
            position,
            radius,
            radius2: radius.powi(2),
            material,
        }
    }
}

impl Entity for Sphere {
    fn collide(&self, ray: &Ray) -> ColliderResult {
        let l = self.position - ray.origin;
        let tca = l.dot(ray.direction);
        if tca < 0.0 {
            return ColliderResult::negative();
        }
        let d2 = l.magnitude2() - tca.powi(2);
        if d2 > self.radius2 {
            return ColliderResult::negative();
        }
        let thc = (self.radius2 - d2).sqrt();
        let pos = ray.origin + (tca - thc) * ray.direction;

        ColliderResult {
            collision: true,
            position: pos,
            normal: InnerSpace::normalize(pos - self.position),
        }
    }

    fn material(&self) -> Option<&Material> {
        Some(&self.material)
    }
}