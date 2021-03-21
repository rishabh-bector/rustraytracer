extern crate cgmath;

use crate::common::*;
use crate::tracer::RayTracer;

use std::io::Write;
use cgmath::Vector3;

pub struct CubemapBehavior {
    // left, right, front, back, down, up
    maps: [image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>; 6],
    mix: f32,
}

impl CubemapBehavior {
    pub fn new(folder: &str, mix: f32) -> CubemapBehavior {
        print!("\nLoading assets...");
        std::io::stdout().flush().unwrap();
        let left = image::open(format!("{}/left.png", folder))
            .unwrap()
            .to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let right = image::open(format!("{}/right.png", folder))
            .unwrap()
            .to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let front = image::open(format!("{}/front.png", folder))
            .unwrap()
            .to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let back = image::open(format!("{}/back.png", folder))
            .unwrap()
            .to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let up = image::open(format!("{}/up.png", folder)).unwrap().to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let down = image::open(format!("{}/down.png", folder))
            .unwrap()
            .to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let maps: [image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>; 6] =
            [left, right, down, up, front, back];
        print!(".\n");
        std::io::stdout().flush().unwrap();
        CubemapBehavior { maps, mix }
    }
}

impl RayBehavior for CubemapBehavior {
    fn compute(
        &self,
        ray: &Ray,
        _world: &World,
        _collision: &ColliderResult,
        _tracer: &RayTracer,
    ) -> Option<Vector3<f32>> {
        let result = cubemap(ray.direction.x, ray.direction.y, ray.direction.z);
        let map = &self.maps[result.0 as usize];
        let mut px = (
            (result.1 * map.dimensions().0 as f32) as u32,
            (result.2 * map.dimensions().1 as f32) as u32,
        );
        if px.0 >= map.dimensions().0 {
            px.0 = map.dimensions().0 - 1;
        }
        if px.1 >= map.dimensions().1 {
            px.1 = map.dimensions().1 - 1;
        }
        let sample = map.get_pixel(px.0, px.1);
        Some(rgb_vec(*sample))
    }

    fn mix(&self) -> f32 {
        self.mix
    }
}

fn cubemap(x: f32, y: f32, z: f32) -> (u32, f32, f32) {
    let abs_x = x.abs();
    let abs_y = y.abs();
    let abs_z = z.abs();

    let is_x_positive = x > 0.0;
    let is_y_positive = y > 0.0;
    let is_z_positive = z > 0.0;

    let mut max_axis = 0 as f32;
    let mut uc = 0 as f32;
    let mut vc = 0 as f32;

    let mut index = 0 as u32;

    // POSITIVE X
    if is_x_positive && abs_x >= abs_y && abs_x >= abs_z {
        // u (0 to 1) goes from +z to -z
        // v (0 to 1) goes from -y to +y
        max_axis = abs_x;
        uc = -z;
        vc = y;
        index = 0;
    }

    // NEGATIVE X
    if !is_x_positive && abs_x >= abs_y && abs_x >= abs_z {
        // u (0 to 1) goes from -z to +z
        // v (0 to 1) goes from -y to +y
        max_axis = abs_x;
        uc = z;
        vc = y;
        index = 1;
    }

    // POSITIVE Y
    if is_y_positive && abs_y >= abs_x && abs_y >= abs_z {
        // u (0 to 1) goes from -x to +x
        // v (0 to 1) goes from +z to -z
        max_axis = abs_y;
        uc = x;
        vc = -z;
        index = 2;
    }

    // NEGATIVE Y
    if !is_y_positive && abs_y >= abs_x && abs_y >= abs_z {
        // u (0 to 1) goes from -x to +x
        // v (0 to 1) goes from -z to +z
        max_axis = abs_y;
        uc = x;
        vc = z;
        index = 3;
    }

    // POSITIVE Z
    if is_z_positive && abs_z >= abs_x && abs_z >= abs_y {
        // u (0 to 1) goes from -x to +x
        // v (0 to 1) goes from -y to +y
        max_axis = abs_z;
        uc = x;
        vc = y;
        index = 4;
    }

    // NEGATIVE Z
    if !is_z_positive && abs_z >= abs_x && abs_z >= abs_y {
        // u (0 to 1) goes from +x to -x
        // v (0 to 1) goes from -y to +y
        max_axis = abs_z;
        uc = -x;
        vc = y;
        index = 5;
    }

    // Convert range from -1 to 1 to 0 to 1
    let u = 0.5 * (uc / max_axis + 1.0);
    let v = 0.5 * (vc / max_axis + 1.0);

    (index, u, v)
}