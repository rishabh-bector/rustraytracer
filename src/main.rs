extern crate cgmath;
extern crate image;
extern crate pbr;

pub mod common;
pub mod tracer;
pub mod material;
pub mod behavior;
pub mod geometry;
pub mod lighting;

use common::*; 
use tracer::*;
use material::*;
use geometry::model;
use geometry::sphere;

use anyhow::Result;
use cgmath::{Vector3, Point3};

fn main() -> Result<()> {
    println!("MAIN!");

    let raytracer = RayTracer::new_default_renderer((1600, 900));
    let mut world = RayTracer::new_empty_world("./cubemaps/hd_blue_sunset");

    let mat1 = Material::new_lambert_material(color_vec(100, 100, 200), 0.8, 1.0, 0.01, 0.1, 20);
    let mat2 = Material::new_lambert_material(color_vec(0, 0, 0), 0.8, 0.0, 1.0, 0.1, 20);
    let sphere = sphere::Sphere::new(
        Point3 {
            x: -3.0,
            y: 0.0,
            z: 5.0,
        },
        1.0,
        mat1,
    );
    let sphere2 = sphere::Sphere::new(
        Point3 {
            x: 2.0,
            y: 0.0,
            z: 8.0,
        },
        1.0,
        mat2,
    );


    // let bounded_box = AABB::new(
    //     Vector3{x: 1.0, y: 1.0, z: 3.0}, 
    //     Vector3{x: 1.25, y: 1.25, z: 3.25}, 
    //     Material::new_lambert_material(color_vec(100, 100, 20), 0.5, 0.5, 0.0, 0.5, 1),
    // );

    let burger = model::Model::new(
        "./obj/ufo_fix.obj",
        Material::new_lambert_material(color_vec(100, 100, 50), 1.0, 1.0, 0.0, 0.1, 20),
        Point3 {x: 0.0, y: 30.0, z: 70.0},
        Vector3 {x: 1.0, y: -1.0, z: 1.0}
    );

    world.entities.push(Box::new(sphere));
    world.entities.push(Box::new(sphere2));
    world.entities.push(Box::new(burger));

    raytracer.render("./bruh.png".to_owned(), world);
    Ok(())
}

// Todo:
// - do multiple ray averages per pixel as an option (anti-aliasing)
// - do more advanced materials, shadows, reflections, refractions
// - make this a published rust crate with instructions on how to use it
// - add more ray collider shapes like cubes, try blending between these like Sebastian Lague
// - try adding in OBJ file support by creating a triangle ray collider
// - do manual animations (.mp4 generation) using output images calculated by setting animation keyframes (moving camera, etc)
// - add in post-processing effects
// - volumes

// Resources:
// https://raytracing.github.io/books/RayTracingTheNextWeek.html
// https://www.realtimerendering.com/raytracing/Ray%20Tracing%20in%20a%20Weekend.pdf
// https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-ray-tracing
// https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-ray-tracing/ray-tracing-practical-example
// https://blog.scottlogic.com/2020/03/10/raytracer-how-to.html
// https://bheisler.github.io/post/writing-raytracer-in-rust-part-1/