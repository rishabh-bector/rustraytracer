extern crate cgmath;

use crate::common::*;
use crate::material::*;
use crate::lighting::*;

use cgmath::{Vector3, InnerSpace};
use std::time;

pub struct RayTracer {
    settings: RenderSettings,
    camera: Camera,
}

pub struct RenderSettings {
    image_size: (u32, u32),
}

pub struct Camera {
    size: (f32, f32),
    lense_factor: (f32, f32),
    position: Vector3<f32>,
}

impl RayTracer {
    pub fn new_default_renderer(size: (u32, u32)) -> RayTracer {
        RayTracer {
            settings: RenderSettings { image_size: size },
            camera: Camera {
                size: (size.0 as f32 / 10.0, size.1 as f32 / 10.0),
                lense_factor: (1.0 as f32, 1.0 as f32),
                position: Vector3 {
                    x: 0 as f32,
                    y: 0 as f32,
                    z: 0 as f32,
                },
            },
        }
    }

    pub fn new_empty_world(skybox: &str) -> World {
        let entities: Vec<Box<dyn Entity>> = Vec::new();
        let sun = DirectionalLight::new(
            Vector3 {
                x: 1.0,
                y: -0.5,
                z: 1.0,
            },
            color_vec(230, 230, 230),
            2.0,
        );

        let light_sources: Vec<Box<dyn LightSource>> = vec![Box::new(sun)];

        let sky = Material::new_sky_material(skybox);

        World {
            entities,
            light_sources,
            sky,
            ambient: 0.15,
        }
    }

    pub fn render(&self, output: String, world: World) {
        println!("Rendering...");
        let timer = time::Instant::now();

        let mut img =
            image::ImageBuffer::new(self.settings.image_size.0, self.settings.image_size.1);

        let lense_pos = self.camera.position
            + Vector3 {
                x: 0.0,
                y: 0.0,
                z: 75.0,
            };
        let lense_size = (
            self.camera.size.0 * self.camera.lense_factor.0,
            self.camera.size.1 * self.camera.lense_factor.1,
        );

        let camera_ll = self.camera.position
            - Vector3 {
                x: self.camera.size.0 / 2.0,
                y: self.camera.size.1 / 2.0,
                z: 0.0,
            };
        let camera_h = Vector3 {
            x: self.camera.size.0,
            y: 0.0,
            z: 0.0,
        };
        let camera_v = Vector3 {
            x: 0.0,
            y: self.camera.size.1,
            z: 0.0,
        };

        let lense_ll = lense_pos
            - Vector3 {
                x: lense_size.0 / 2.0,
                y: lense_size.1 / 2.0,
                z: 0.0,
            };
        let lense_h = Vector3 {
            x: lense_size.0,
            y: 0.0,
            z: 0.0,
        };
        let lense_v = Vector3 {
            x: 0.0,
            y: lense_size.1,
            z: 0.0,
        };
        for (x, y, p) in img.enumerate_pixels_mut() {
            let camera_point = camera_ll
                + (x as f32 / self.settings.image_size.0 as f32) * camera_h
                + (y as f32 / self.settings.image_size.1 as f32) * camera_v;
            let camera_point = self.camera.position;

            let lense_point = lense_ll
                + (x as f32 / self.settings.image_size.0 as f32) * lense_h
                + (y as f32 / self.settings.image_size.1 as f32) * lense_v;
            let dir = InnerSpace::normalize(lense_point - camera_point);

            // Transform camera
            // let mtx = cgmath::Matrix4::from_translation(Vector3{})

            let ray = Ray {
                origin: camera_point,
                direction: dir,
                bounce: 0,
            };

            *p = vec_rgb(self.cast(&ray, &world));

            // if x == 0 && y == 0 {
            //     println!("{:?}", ray.direction);
            // }
        }

        match img.save(output) {
            Ok(_) => println!("Saved!"),
            Err(e) => println!("{}", e),
        };
        println!("\n");

        let duration = timer.elapsed();
        println!("Finished in {}ms", duration.as_millis());
    }

    pub fn cast(&self, ray: &Ray, world: &World) -> Vector3<f32> {
        let mut min_distance = f32::MAX;
        let mut closest_collision = None;
        for entity in world.entities.iter() {
            let ent = entity.as_ref();
            let result = ent.collide(ray);
            if result.collision {
                let collision_dist = (result.position - ray.origin).magnitude2();
                if collision_dist < min_distance {
                    min_distance = collision_dist;
                    closest_collision = Some((result, ent));
                }
            }
        }

        if let Some((result, ent)) = closest_collision {
            let material = ent.material().unwrap();

            let mut final_color: Vector3<f32> = material.color * world.ambient;
            for behavior in material.shaders.iter() {
                match behavior.as_ref().compute(ray, world, &result, self) {
                    Some(color) => {
                        final_color = final_color + color * behavior.mix();
                    }
                    None => continue,
                }
            }
            return final_color;
        }
        
        // Sky
        match world.sky.shaders[0].compute(ray, world, &ColliderResult::negative(), self) {
            Some(color) => color,
            None => color_vec(178, 222, 236),
        }
    }
}