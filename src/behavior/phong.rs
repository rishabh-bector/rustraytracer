extern crate cgmath;

use crate::common::{Ray, RayBehavior, World, ColliderResult};
use crate::tracer::RayTracer;
use crate::lighting::LightRay;

use cgmath::{Vector3, InnerSpace};

pub struct PhongBehavior {
    mix: f64,
    alpha: i32,
}

impl PhongBehavior {
    pub fn new(mix: f64, alpha: i32) -> PhongBehavior {
        PhongBehavior{mix, alpha}
    }
}

impl RayBehavior for PhongBehavior {
    fn mix(&self) -> f64 {
        self.mix
    }

    fn compute(
        &self,
        ray: &Ray,
        world: &World,
        collision: &ColliderResult,
        _tracer: &RayTracer,
    ) -> Option<Vector3<f64>> {
        let mut result = Vector3 {
            x: 0.,
            y: 0.,
            z: 0.,
        };
        for light_source in world.light_sources.iter() {
            if light_source.visible(collision.position, collision.normal, world) {
                let LightRay { power, direction } =
                    light_source.illuminate(collision.position, collision.normal);
                let ray_bisector = (-direction - ray.direction).normalize();
                let power = power * ray_bisector.dot(collision.normal).max(0.).powi(self.alpha);
                result += light_source.color() * power;
            }
        }
        Some(result)
    }
}
