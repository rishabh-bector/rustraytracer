extern crate cgmath;

use crate::material::Material;
use crate::common::{Entity, ColliderResult, Ray};

use cgmath::{Vector3, InnerSpace};

pub struct AABB {
    min: Vector3<f32>,
    max: Vector3<f32>,
    material: Material,
}

impl AABB {
    pub fn new(min: Vector3<f32>, max: Vector3<f32>, material: Material) -> AABB {
        AABB {
            min, max, material,
        }
    }
}

impl Entity for AABB {
    fn collide(&self, ray: &Ray) -> ColliderResult {
        let mut tmin = (self.min.x - ray.origin.x) / ray.direction.x; 
        let mut tmax = (self.max.x - ray.origin.x) / ray.direction.x; 
        if tmin > tmax {
            let t = tmin;
            tmin = tmax;
            tmax = t;
        } 
        let mut tymin = (self.min.y - ray.origin.y) / ray.direction.y; 
        let mut tymax = (self.max.y - ray.origin.y) / ray.direction.y; 
 
        if tymin > tymax {
            let t = tymin;
            tymin = tymax;
            tymax = t;
        }

        if (tmin > tymax) || (tymin > tmax) {
            return ColliderResult::negative();
        }
 
        if tymin > tmin {
            tmin = tymin; 
        }
        
        if tymax < tmax {
            tmax = tymax;
        } 

        let mut tzmin = (self.min.z - ray.origin.z) / ray.direction.z; 
        let mut tzmax = (self.max.z - ray.origin.z) / ray.direction.z; 
 
        if tzmin > tzmax {
            let t = tzmin;
            tzmin = tzmax;
            tzmax = t;
        }

        if (tmin > tzmax) || (tzmin > tmax) {
            return ColliderResult::negative(); 
 }
        if (tzmin > tmin) {
            tmin = tzmin; 
        }
    
        if (tzmax < tmax) {
            tmax = tzmax; 
        }

        let position = ray.parameterize(tmin);
        let delta = 0.01;
        let check = |a: f32, b: f32, delta: f32| -> bool {a >= b-delta && a <= b+delta};
        let mut normal = Vector3{x: 0.0, y: 0.0, z: 0.0};
        if check(position.x, self.min.x, delta) {
            normal.x = 1.0;
        } else if check(position.x, self.max.x, delta) {
            normal.x = -1.0;
        } else if check(position.y, self.min.y, delta) {
            normal.y = 1.0;
        } else if check(position.y, self.max.x, delta) {
            normal.y = -1.0;
        } else if check(position.z, self.min.z, delta) {
            normal.z = 1.0;
        } else if check(position.z, self.max.x, delta) {
            normal.z = -1.0;
        }
 
        ColliderResult{
            collision: true, 
            normal,
            position,
        }
    }

    fn material(&self) -> Option<&Material> {
        Some(&self.material)
    }
}