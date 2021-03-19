extern crate cgmath;

use crate::material::Material;
use crate::tracer::RayTracer;
use crate::lighting::LightSource;
use crate::geometry::aabb::AABB;

use cgmath::{Vector3, Point3, EuclideanSpace};

pub struct World {
    pub entities: Vec<Box<dyn Entity>>,
    pub light_sources: Vec<Box<dyn LightSource>>,
    pub sky: Material,
    pub ambient: f32,
}

pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub bounce: u32,
}

impl Ray {
    pub fn parameterize(&self, t: f32) -> Vector3<f32> {
        (self.origin + self.direction * t).to_vec()
    }
}

pub trait RayBehavior {
    fn compute(
        &self,
        ray: &Ray,
        world: &World,
        collision: &ColliderResult,
        tracer: &RayTracer,
    ) -> Option<Vector3<f32>>;

    fn mix(&self) -> f32;
}

pub trait Entity {
    fn collide(&self, ray: &Ray) -> ColliderResult;
    fn material(&self) -> Option<&Material>;
    fn bounding_box(&self) -> AABB;
    fn position(&self) -> Point3<f32>;
}

pub struct ColliderResult {
    pub collision: bool,
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
}

impl ColliderResult {
    pub fn negative() -> ColliderResult {
        ColliderResult {
            collision: false,
            position: Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            normal: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }
}

pub fn vec_rgb(v: Vector3<f32>) -> image::Rgb<u8> {
    image::Rgb([
        (v.x * 255.0) as u8,
        (v.y * 255.0) as u8,
        (v.z * 255.0) as u8,
    ])
}

pub fn rgb_vec(i: image::Rgb<u8>) -> Vector3<f32> {
    color_vec(i[0], i[1], i[2])
}

pub fn color_vec(r: u8, g: u8, b: u8) -> Vector3<f32> {
    Vector3 {
        x: (r as f32) / 255.0,
        y: (g as f32) / 255.0,
        z: (b as f32) / 255.0,
    }
}

pub fn lerp(v1: Vector3<f32>, v2: Vector3<f32>, amount: f32) -> Vector3<f32> {
    Vector3 {
        x: v1.x + (v2.x - v1.x) * amount,
        y: v1.y + (v2.y - v1.y) * amount,
        z: v1.z + (v2.z - v1.z) * amount,
    }
}

pub fn vector3(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3{x, y, z}
}