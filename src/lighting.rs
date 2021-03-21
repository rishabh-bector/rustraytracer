extern crate cgmath;

use crate::common::*;

use cgmath::{InnerSpace, Point3, Vector3};

pub struct LightRay {
    pub power: f32,
    pub direction: Vector3<f32>,
}

pub trait LightSource: Sync + Send {
    fn illuminate(&self, pos: Point3<f32>, normal: Vector3<f32>) -> LightRay;
    fn visible(&self, pos: Point3<f32>, normal: Vector3<f32>, world: &World) -> bool;
    fn color(&self) -> Vector3<f32>;
}

pub struct DirectionalLight {
    direction: Vector3<f32>,
    color: Vector3<f32>,
    intensity: f32,
}

impl DirectionalLight {
    pub fn new(direction: Vector3<f32>, color: Vector3<f32>, intensity: f32) -> Self {
        let direction = direction.normalize();
        Self {
            direction,
            color,
            intensity,
        }
    }
}

impl LightSource for DirectionalLight {
    fn illuminate(&self, _pos: Point3<f32>, _normal: Vector3<f32>) -> LightRay {
        LightRay {
            power: self.intensity,
            direction: self.direction,
        }
    }

    fn visible(&self, _pos: Point3<f32>, normal: Vector3<f32>, _world: &World) -> bool {
        normal.dot(self.direction) < 0.
    }

    fn color(&self) -> Vector3<f32> {
        self.color
    }
}

pub struct PointLight {
    position: Point3<f32>,
    color: Vector3<f32>,
    brightness: f32,
    attenuation: f32,
}

impl LightSource for PointLight {
    fn illuminate(&self, pos: Point3<f32>, _normal: Vector3<f32>) -> LightRay {
        let direction = pos - self.position;
        let distance2 = direction.magnitude2();
        let direction = direction.normalize();
        LightRay {
            power: self.brightness / (self.attenuation * distance2),
            direction,
        }
    }

    fn visible(&self, pos: Point3<f32>, _normal: Vector3<f32>, world: &World) -> bool {
        let direction = self.position - pos;
        let ray = Ray {
            origin: pos,
            direction,
            bounce: 0,
        };
        for ent in world.entities.iter() {
            let result = ent.collide(&ray);
            if !result.collision {
                return true;
            }
        }
        false
    }

    fn color(&self) -> Vector3<f32> {
        self.color
    }
}