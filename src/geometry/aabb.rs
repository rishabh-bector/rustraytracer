extern crate cgmath;

use std::{ ops::Deref };

use crate::material::Material;
use crate::common::{Entity, ColliderResult, Ray};

use cgmath::{EuclideanSpace, Point3, Vector3};

#[derive(Copy, Clone)]
pub struct AABB {
    pub min: Point3<f64>,
    pub max: Point3<f64>
}

impl AABB {
    pub fn new(min: Point3<f64>, max: Point3<f64>) -> AABB {
        AABB { min, max }
    }

    pub fn default() -> Self {
        AABB::new(Point3 {x: 0., y: 0., z: 0.}, Point3 {x: 0., y: 0., z: 0.})
    }

    pub fn from_entities<T: Entity + ?Sized> (entities: impl Iterator<Item = impl Deref<Target = T>>) -> Self {
        let mut min = Point3{x: std::f64::MAX, y: std::f64::MAX, z: std::f64::MAX};
        let mut max = Point3{x: std::f64::MIN, y: std::f64::MIN, z: std::f64::MIN};
        for entity in entities {
            let bb = entity.bounding_box();
            if bb.min.x < min.x { min.x = bb.min.x; }
            if bb.min.y < min.y { min.y = bb.min.y; }
            if bb.min.z < min.z { min.z = bb.min.z; }

            if bb.max.x > max.x { max.x = bb.max.x; }
            if bb.max.y > max.y { max.y = bb.max.y; }
            if bb.max.z > max.z { max.z = bb.max.z; }
        }
        AABB { min, max }
    }

    pub fn contains (&self, point: &Point3<f64>) -> bool {
        if point.x > self.max.x || point.x < self.min.x { return false; }
        if point.y > self.max.y || point.y < self.min.y { return false; }
        if point.z > self.max.z || point.z < self.min.z { return false; }
        true
    }
}

impl Entity for AABB {
    fn collide(&self, ray: &Ray) -> ColliderResult {

        let mut candidate_dist = [0., 0., 0.];
        let hit_point;
        let inside = self.contains(&ray.origin);
 
        for i in 0..3 {
            if ray.origin[i] < self.min[i] {
                candidate_dist[i] = self.min[i] - ray.origin[i];
                if ray.direction[i] < 0. { return ColliderResult::negative(); }
            } else if ray.origin[i] > self.max[i] {
                candidate_dist[i] = self.max[i] - ray.origin[i];
                if ray.direction[i] > 0. { return ColliderResult::negative(); }
            } else {
                if inside {
                    if ray.direction[i] > 0. {
                        candidate_dist[i] = self.max[i] - ray.origin[i];
                    } else {
                        candidate_dist[i] = self.min[i] - ray.origin[i];
                    }
                } else {
                    candidate_dist[i] = -ray.direction[i];
                }
            }
        }

        let one_over_dir = 1. / ray.direction;

        let times = (0..3).map(|i| 
                if !one_over_dir[i].is_finite() {
                    if candidate_dist[i] == 0. {-1.}
                    else {std::f64::MAX}
                } else { candidate_dist[i] * one_over_dir[i] });
        if inside {
            hit_point = ray.parameterize(
                times.min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
            );
        } else {
            hit_point = ray.parameterize(
                times.max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
            );
            if !self.contains(&Point3::from_vec(hit_point + Vector3 {x: 0.001, y: 0.001, z: 0.001})) {
                return ColliderResult::negative();
            }
        }

        ColliderResult {
            normal: Vector3 {x: 0., y: 0., z: 0.},
            collision: true,
            material: None,
            position: Point3::from_vec(hit_point)
        }
    }

    fn bounding_box(&self) -> AABB {
        return AABB {
            min: self.min,
            max: self.max
        }
    }

    fn material(&self) -> Option<&Material> { None }
    
    fn position(&self) -> Point3<f64> {
        let pos = self.min + (self.max - self.min) / 2.;
        Point3 {x: pos.x, y: pos.y, z: pos.z}
    }

    fn translate(&mut self, vec: Vector3<f64>) {
        self.min += vec;
        self.max += vec;
    }
}