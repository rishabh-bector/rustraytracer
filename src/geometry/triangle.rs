extern crate cgmath;

use crate::material::Material;
use crate::common::{Entity, ColliderResult, Ray};

use cgmath::{Vector3, InnerSpace};

pub struct Triangle {
    v0: Vector3<f32>,
    v1: Vector3<f32>,
    v2: Vector3<f32>,
    normal: Vector3<f32>,
}

impl Triangle {
    pub fn new(v0: Vector3<f32>, v1: Vector3<f32>, v2: Vector3<f32>, normal: Vector3<f32>) -> Triangle {
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
        let culling = false;
        let kEpsilon = 0.01;

        let v0v1 = self.v1 - self.v0;
        let v0v2 = self.v2 - self.v0;
        let area2 = self.normal.magnitude2();

        let NdotRayDirection = self.normal.dot(ray.direction);
        if NdotRayDirection.abs() < kEpsilon {
            return ColliderResult::negative();
        }

        let d = self.normal.dot(self.v0);
        let t = (self.normal.dot(ray.origin) + d) / NdotRayDirection;
        if t < 0.0 {
            return ColliderResult::negative();
        }

        let P = ray.origin + t * ray.direction;
        let mut C: Vector3<f32>;

        let edge0 = self.v1 - self.v0;
        let vp0 = P - self.v0;
        C = edge0.cross(vp0);
        if self.normal.dot(C) < 0.0 {
            return ColliderResult::negative();
        }

        let edge1 = self.v2 - self.v1;
        let vp1 = P - self.v1;
        C = edge1.cross(vp1);
        if self.normal.dot(C) < 0.0 {
            return ColliderResult::negative();
        }

        let edge2 = self.v0 - self.v2;
        let vp2 = P - self.v2;
        C = edge2.cross(vp2);
        if self.normal.dot(C) < 0.0 {
            return ColliderResult::negative();
        }

        ColliderResult {
            collision: true,
            position: ray.parameterize(t),
            normal: self.normal,
        }
    }

    fn material(&self) -> Option<&Material> {
        None
    }
}