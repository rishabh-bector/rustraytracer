extern crate cgmath;

use crate::material::Material;
use crate::common::{Entity, ColliderResult, Ray};
use crate::geometry::aabb::AABB;

use cgmath::{Vector3, InnerSpace, Point3};

pub struct Triangle {
    pub v0: Point3<f32>,
    pub v1: Point3<f32>,
    pub v2: Point3<f32>,
    pub normal: Vector3<f32>,
}

impl Clone for Triangle {
    fn clone(&self) -> Triangle {
        Triangle {
            v0: self.v0.clone(),
            v1: self.v1.clone(),
            v2: self.v2.clone(),
            normal: self.normal.clone()
        }
    }
}

impl Triangle {
    pub fn new(v0: Point3<f32>, v1: Point3<f32>, v2: Point3<f32>, normal: Vector3<f32>) -> Triangle {
        Triangle { v0, v1, v2, normal }
    } 

    pub fn compute_normal(&mut self) {
        let v0v1 = self.v1 - self.v0;
        let v0v2 = self.v2 - self.v0;
        self.normal = v0v1.cross(v0v2).normalize();
    }
}

impl Entity for Triangle {
    fn collide(&self, ray: &Ray) -> ColliderResult {

        // Möller–Trumbore intersection algorithm

        const EPSILON: f32 = 0.0000001;
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);
        if a > -EPSILON && a < EPSILON {
            return ColliderResult::negative();
        }
        let f = 1.0/a;
        let s = ray.origin - self.v0;
        let u = f * s.dot(h);
        if u < 0.0 || u > 1.0 {
            return ColliderResult::negative();
        }
        let q = s.cross(edge1);
        let v = f * ray.direction.dot(q);
        if v < 0.0 || u + v > 1.0 {
            return ColliderResult::negative();
        }
        let t = f * edge2.dot(q);
        if t > EPSILON {
            return ColliderResult{
                collision: true,
                position: ray.origin + ray.direction * t,
                normal: self.normal,
            }
        }
        return ColliderResult::negative();
    }

    fn bounding_box(&self) -> AABB {
        AABB::new(
            Point3 {x: self.v0.x.min(self.v1.x.min(self.v2.x)), y: self.v0.y.min(self.v1.y.min(self.v2.y)), z: self.v0.z.min(self.v1.z.min(self.v2.z))},
            Point3 {x: self.v0.x.max(self.v1.x.max(self.v2.x)), y: self.v0.y.max(self.v1.y.max(self.v2.y)), z: self.v0.z.max(self.v1.z.max(self.v2.z))}
        )
    }

    fn position(&self) -> Point3<f32> {
        Point3 {
            x: (self.v0.x + self.v1.x + self.v2.x) / 3.,
            y: (self.v0.y + self.v1.y + self.v2.y) / 3.,
            z: (self.v0.z + self.v1.z + self.v2.z) / 3.
        }
    }

    fn material(&self) -> Option<&Material> {
        None
    }
}