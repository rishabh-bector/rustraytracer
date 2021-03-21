extern crate cgmath;

use std::{rc::Rc, sync::Arc};

use crate::common::{RayBehavior, color_vec};
use crate::behavior::cubemap::CubemapBehavior;
use crate::behavior::lambert::LambertBehavior;
use crate::behavior::phong::PhongBehavior;
use crate::behavior::reflection::ReflectionBehavior;

use cgmath::{Vector3};

#[derive(Clone)]
pub struct Material {
    pub shaders: Vec<Arc<dyn RayBehavior>>,
    pub color: Vector3<f32>,
}

impl Material { 
    pub fn new_lambert_material(
        color: Vector3<f32>,
        albedo: f32,
        lambert: f32,
        reflective: f32,
        phong: f32,
        alpha: i32,
    ) -> Material {
        let lambert_behavior = LambertBehavior::new(albedo, lambert, color);
        let ref_be = ReflectionBehavior::new(reflective);
        let phong_behavior = PhongBehavior::new(phong, alpha);
        let mut shaders: Vec<Arc<dyn RayBehavior>> = Vec::new();
        shaders.push(Arc::new(lambert_behavior));
        shaders.push(Arc::new(ref_be));
        shaders.push(Arc::new(phong_behavior));
        Material { shaders, color }
    }

    pub fn new_sky_material(cubemap_folder: &str) -> Material {
        let cubemap_behavior = CubemapBehavior::new(cubemap_folder, 1.0);
        let mut shaders: Vec<Arc<dyn RayBehavior>> = Vec::new();
        shaders.push(Arc::new(cubemap_behavior));
        Material {
            shaders,
            color: color_vec(0, 0, 0),
        }
    }
}