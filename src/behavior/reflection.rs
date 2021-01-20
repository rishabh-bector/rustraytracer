extern crate cgmath;

use crate::common::{Ray, RayBehavior, World, ColliderResult};
use crate::tracer::RayTracer;

use cgmath::{Vector3, InnerSpace};

pub struct ReflectionBehavior {
    mix: f32,
}

impl ReflectionBehavior {
    pub fn new(mix: f32) -> ReflectionBehavior {
        ReflectionBehavior { mix }
    }
}

impl RayBehavior for ReflectionBehavior {
    fn compute(
        &self,
        ray: &Ray,
        world: &World,
        collision: &ColliderResult,
        tracer: &RayTracer,
    ) -> Option<Vector3<f32>> {
        if ray.bounce > 2 {
            return None;
        };
        let reflected = Ray {
            origin: collision.position + collision.normal * 0.3,
            direction: InnerSpace::normalize(reflect(ray.direction, collision.normal)),
            bounce: ray.bounce + 1,
        };
        Some(tracer.cast(&reflected, world))
    }

    fn mix(&self) -> f32 {
        self.mix
    }
}

fn reflect(d: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
    d - (n * (n.dot(d)) * 2.0)
}