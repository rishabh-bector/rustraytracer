extern crate cgmath;

use crate::material::Material;
use crate::common::{Entity, ColliderResult, Ray};
use crate::common;

use cgmath::{Vector3, InnerSpace, Point3};

pub struct Triangle {
    v0: Point3<f32>,
    v1: Point3<f32>,
    v2: Point3<f32>,
    normal: Vector3<f32>,
}

impl Triangle {
    pub fn new(v0: Point3<f32>, v1: Point3<f32>, v2: Point3<f32>, normal: Vector3<f32>) -> Triangle {
        Triangle { v0, v1, v2, normal }
    } 

    pub fn compute_normal(&mut self) {
        let v0v1 = self.v1 - self.v0;
        let v0v2 = self.v2 - self.v0;
        self.normal = v0v1.cross(v0v2);
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
        let s = common::vec2point(ray.origin) - self.v0;
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

    fn material(&self) -> Option<&Material> {
        None
    }
}