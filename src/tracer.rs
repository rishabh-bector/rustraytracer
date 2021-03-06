extern crate cgmath;

use crate::common::*;
use crate::material::*;
use crate::lighting::*;

use cgmath::{Vector3, Point3, InnerSpace};
use image::{ImageBuffer, Rgb};
use std::{sync::Arc, thread, time};

pub struct RayTracer {
    settings: RenderSettings,
    camera: Camera,
}

pub struct RenderSettings {
    image_size: (u32, u32),
}

pub struct Camera {
    size: (f64, f64),
    lens_factor: (f64, f64),
    position: Point3<f64>,
}

impl RayTracer {
    pub const fn default() -> Self {
        RayTracer {
            settings: RenderSettings {image_size: (0, 0)},
            camera: Camera {
                size: (0., 0.),
                lens_factor: (0., 0.),
                position: Point3 {x: 0., y: 0., z: 0.}
            }
        }
    }

    pub fn new_default_renderer(size: (u32, u32)) -> RayTracer {
        RayTracer {
            settings: RenderSettings { image_size: size },
            camera: Camera {
                size: (160.0, 90.0),
                lens_factor: (1., 1.),
                position: Point3 {
                    x: 0.,
                    y: 0.,
                    z: 0.,
                },
            },
        }
    }

    pub fn new_empty_world(skybox: &str) -> World {
        let entities: Vec<Box<dyn Entity>> = Vec::new();
        let sun = DirectionalLight::new(
            Vector3 {
                x: 1.0,
                y: -1.0,
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

    pub fn render(self, output: String, world: World) {
        println!("Rendering...");
        let timer = time::Instant::now();

        let mut img: ImageBuffer<image::Rgb<u8>, Vec<_>> = 
            image::ImageBuffer::new(self.settings.image_size.0, self.settings.image_size.1);

        let lense_pos = self.camera.position
            + Vector3 {
                x: 0.0,
                y: 0.0,
                z: 75.0,
            };
        let lense_size = (
            self.camera.size.0 * self.camera.lens_factor.0,
            self.camera.size.1 * self.camera.lens_factor.1,
        );

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
        
        let num_pixels = img.enumerate_pixels().len() as i32;

        let arc_world = Arc::new(world);
        let arc_self = Arc::new(self);

        let num_threads = 12_usize;
        let mut rays: Vec<Vec<_>> = (0..num_threads).into_iter().map(|_|Vec::new()).collect();
        let mut threads = Vec::new();

        let chunk_size = num_pixels as usize / num_threads;

        for (i, (x, y, p)) in img.enumerate_pixels_mut().into_iter().enumerate() {
            let thread_index = i / chunk_size;

            let camera_point = arc_self.camera.position;

            let lense_point = lense_ll
                + (x as f64 / arc_self.settings.image_size.0 as f64) * lense_h
                + (y as f64 / arc_self.settings.image_size.1 as f64) * lense_v;
            let dir = InnerSpace::normalize(lense_point - camera_point);

            // Transform camera
            // let mtx = cgmath::Matrix4::from_translation(Vector3{})

            let ray = Ray {
                origin: camera_point,
                direction: dir,
                bounce: 0,
            };

            let arc_world = arc_world.clone();
            let arc_self = arc_self.clone();

            struct Bad(*mut Rgb<u8>);
            unsafe impl Send for Bad {}
            let p = Bad(p);

            rays[thread_index].push(
                move || unsafe {*p.0 = vec_rgb(arc_self.cast(&ray, &arc_world))}
            );
        }

        for ray in rays {
            threads.push(thread::spawn(
                move || ray.into_iter().for_each(|a| {a();})
            ));
        }

        let mut i = 0f32;
        for thread in threads {
            thread.join().unwrap();
            println!("Progress: {}%", i * 100.0);
            i += 1. / num_threads as f32;
        }

        match img.save(output) {
            Ok(_) => println!("Saved!"),
            Err(e) => println!("{}", e),
        };
        println!("\n");

        let duration = timer.elapsed();
        println!("Finished in {}ms", duration.as_millis());
    }

    pub fn cast(&self, ray: &Ray, world: &World) -> Vector3<f64> {
        let mut min_distance = f64::MAX;
        let mut closest_collision = None;
        for entity in world.entities.iter() {
            let ent = entity.as_ref();
            let result = ent.collide(ray);
            if result.collision {
                let collision_dist = (result.position - ray.origin).magnitude2();
                if collision_dist < min_distance {
                    min_distance = collision_dist;
                    closest_collision = Some(result);
                }
            }
        }

        if let Some(result) = closest_collision {
            let material = result.material.as_ref().unwrap();
            let mut final_color: Vector3<f64> = material.color * world.ambient;
            for behavior in material.shaders.iter() {
                match behavior.as_ref().compute(ray, world, &result, self) {
                    Some(color) => {
                        final_color += color * behavior.mix();
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
