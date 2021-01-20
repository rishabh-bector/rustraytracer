extern crate cgmath;

use crate::common::{Ray, RayBehavior, World, ColliderResult};
use crate::tracer::RayTracer;
use crate::lighting::LightRay;

use cgmath::{Vector3, InnerSpace};

pub struct LambertBehavior {
    albedo: f32,
    mix: f32,
    color: Vector3<f32>,
}

impl LambertBehavior {
    pub fn new(albedo: f32, mix: f32, color: Vector3<f32>) -> LambertBehavior {
        LambertBehavior { albedo, mix, color }
    }
}

impl RayBehavior for LambertBehavior {
    fn compute(
        &self,
        _ray: &Ray,
        world: &World,
        collision: &ColliderResult,
        _tracer: &RayTracer,
    ) -> Option<Vector3<f32>> {
        let mut result = Vector3 {
            x: 0.,
            y: 0.,
            z: 0.,
        };
        for light_source in world.light_sources.iter() {
            if light_source.visible(collision.position, collision.normal, world) {
                let LightRay { power, direction } =
                    light_source.illuminate(collision.position, collision.normal);
                let power =
                    power * (self.albedo / std::f32::consts::PI) * -collision.normal.dot(direction);
                let power = power.max(0.);
                result += self.color * power;
            }
        }
        Some(result)
    }

    fn mix(&self) -> f32 {
        self.mix
    }
}