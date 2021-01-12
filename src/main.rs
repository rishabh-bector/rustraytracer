extern crate image;
extern crate cgmath;

use std::time;
use cgmath::{Vector3, InnerSpace};
use std::io::Write;

trait Entity {
    fn collide(&self, ray: &Ray) -> ColliderResult;
    fn material(&self) -> &Material;
}

trait RayBehavior {
    fn compute(
        &self, 
        ray: &Ray,
        world: &World,
        collision: &ColliderResult,
        tracer: &RayTracer,
    ) -> Option<Vector3<f32>>;

    fn mix(&self) -> f32;
}

struct ColorBehavior {
    color: Vector3<f32>,
    mix: f32,
}

impl ColorBehavior {
    fn new(color: Vector3<f32>, mix: f32) -> ColorBehavior {
        ColorBehavior { color, mix }
    }
}

impl RayBehavior for ColorBehavior {
    fn compute(
        &self, 
        _ray: &Ray,
        _world: &World,
        _collision: &ColliderResult,
        _tracer: &RayTracer,
    ) -> Option<Vector3<f32>> { Some(self.color) }

    fn mix(&self) -> f32 { self.mix }
}

struct LambertBehavior {
    albedo: f32,
    mix: f32,
}

impl LambertBehavior {
    fn new(albedo: f32, mix: f32) -> LambertBehavior {
        LambertBehavior { albedo, mix }
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
        Some(world.sun.diffuse_directional(collision.normal, self.albedo))
    }

    fn mix(&self) -> f32 { self.mix }
}

struct CubemapBehavior {
    // left, right, front, back, down, up
    maps: [image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>; 6],
    mix: f32,
}

impl CubemapBehavior {
    fn new(folder: &str, mix: f32) -> CubemapBehavior {
        print!("\nLoading assets...");
        std::io::stdout().flush().unwrap();
        let left = image::open(format!("{}/left.png", folder)).unwrap().to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let right = image::open(format!("{}/right.png", folder)).unwrap().to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let front = image::open(format!("{}/front.png", folder)).unwrap().to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let back = image::open(format!("{}/back.png", folder)).unwrap().to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let up = image::open(format!("{}/up.png", folder)).unwrap().to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let down = image::open(format!("{}/down.png", folder)).unwrap().to_rgb();
        print!(".");
        std::io::stdout().flush().unwrap();
        let maps: [image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>; 6] = [left, right, down, up, front, back];
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
        let mut px = ((result.1 * map.dimensions().0 as f32) as u32, (result.2 * map.dimensions().1 as f32) as u32);
        if px.0 >= map.dimensions().0 { px.0 = map.dimensions().0 - 1; }
        if px.1 >= map.dimensions().1 { px.1 = map.dimensions().1 - 1; }
        let sample = map.get_pixel(px.0, px.1);
        Some(rgb_vec(*sample))
    }

    fn mix(&self) -> f32 { self.mix }
}

struct ReflectionBehavior {
    mix: f32,
}

impl ReflectionBehavior {
    fn new(mix: f32) -> ReflectionBehavior {
        ReflectionBehavior{ mix }
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
        if ray.bounce > 1 { return None };
        let reflected = Ray{
            origin: collision.position + collision.normal,
            direction: InnerSpace::normalize(reflect(ray.direction, collision.normal)),
            bounce: ray.bounce + 1,
        };
        Some(tracer.cast(&reflected, world))
    }

    fn mix(&self) -> f32 { self.mix }
}

struct RenderSettings {
    image_size: (u32, u32),
}

struct Material {
    pub shaders: Vec<Box<dyn RayBehavior>>
}

impl Material {
    fn new_color_material(color: Vector3<f32>) -> Material {
        let color_behavior = ColorBehavior::new(color, 1.0);
        let mut shaders: Vec<Box<dyn RayBehavior>> = Vec::new();
        shaders.push(Box::new(color_behavior));
        Material { shaders }
    }

    fn new_lambert_material(color: Vector3<f32>, albedo: f32, mix: f32) -> Material {
        let color_behavior = ColorBehavior::new(color, 1.0);
        let lambert_behavior = LambertBehavior::new(albedo, mix);
        let ref_be = ReflectionBehavior::new(1.0);
        let mut shaders: Vec<Box<dyn RayBehavior>> = Vec::new();
        shaders.push(Box::new(color_behavior));
        shaders.push(Box::new(lambert_behavior));
        shaders.push(Box::new(ref_be));
        Material { shaders }
    }

    fn new_sky_material(cubemap_folder: &str) -> Material {
        let cubemap_behavior = CubemapBehavior::new(cubemap_folder, 1.0);
        let mut shaders: Vec<Box<dyn RayBehavior>> = Vec::new();
        shaders.push(Box::new(cubemap_behavior));
        Material { shaders }
    }
}

struct RayTracer {
    settings: RenderSettings,
    camera: Camera,
}

impl RayTracer {
    fn new_default_renderer(size: (u32, u32)) -> RayTracer {
        RayTracer {
            settings: RenderSettings {
                image_size: size,
            },
            camera: Camera {
                size: (size.0 as f32 / 10.0, size.1 as f32 / 10.0),
                lense_factor: (1.0 as f32, 1.0 as f32),
                position: Vector3{x: 0 as f32, y: 0 as f32, z: 0 as f32},
            },
        }
    }

    fn new_empty_world() -> World {
        let entities: Vec<Box<dyn Entity>> = Vec::new();
        let sun = DirectionalLight {
            direction: Vector3{x: 1.0, y: -0.5, z: -1.0},
            color: color_vec(230, 230, 230),
            intensity: 2.0,
        };

        let sky = Material::new_sky_material("./cubemaps/space");

        World {entities, sun, sky}
    }

    fn render(&self, output: String, world: World) {
        println!("Rendering...");
        let timer = time::Instant::now();

        let mut img = image::ImageBuffer::new(self.settings.image_size.0, self.settings.image_size.1);

        let lense_pos = self.camera.position + Vector3{x: 0.0, y: 0.0, z: 75.0};
        let lense_size = (self.camera.size.0 * self.camera.lense_factor.0, self.camera.size.1 * self.camera.lense_factor.1);

        let camera_ll = self.camera.position - Vector3{x: self.camera.size.0 / 2.0, y: self.camera.size.1 / 2.0, z: 0.0};
        let camera_h = Vector3{x: self.camera.size.0, y: 0.0, z: 0.0};
        let camera_v = Vector3{x: 0.0, y: self.camera.size.1, z: 0.0};
    
        let lense_ll = lense_pos - Vector3{x: lense_size.0 / 2.0, y: lense_size.1 / 2.0, z: 0.0};
        let lense_h = Vector3{x: lense_size.0, y: 0.0, z: 0.0};
        let lense_v = Vector3{x: 0.0, y: lense_size.1, z: 0.0};
        
        for (x, y, p) in img.enumerate_pixels_mut() {
            let camera_point = camera_ll + 
                (x as f32 / self.settings.image_size.0 as f32) * camera_h + 
                (y as f32 / self.settings.image_size.1 as f32) * camera_v;
            let camera_point = self.camera.position;

            let lense_point = lense_ll + 
                (x as f32 / self.settings.image_size.0 as f32) * lense_h + 
                (y as f32 / self.settings.image_size.1 as f32) * lense_v;
            let dir = InnerSpace::normalize(lense_point - camera_point);
            let ray = Ray {
                origin: camera_point,
                direction: dir,
                bounce: 0
            };

            *p = vec_rgb(self.cast(&ray, &world));
    
            if x == 0 && y == 0 {
                println!("{:?}", ray.direction);
            }
        }

        match img.save(output) {
            Ok(_) => println!("Saved!"),
            Err(e) => println!("{}", e), 
        };

        let duration = timer.elapsed();
        println!("Finished in {}ms", duration.as_millis());
    }
    
    fn cast(&self, ray: &Ray, world: &World) -> Vector3<f32> {        
        for entity in world.entities.iter() {
            let ent = entity.as_ref();
            let result = ent.collide(ray);
            if !result.collision { continue }
            let material = ent.material();

            let mut final_color: Vector3<f32> = Vector3{x: -1.0, y: -1.0, z: -1.0};
            for behavior in material.shaders.iter() {
                match behavior.as_ref().compute(ray, world, &result, self) {
                    Some(color) => {
                        if final_color.x == -1.0 { final_color = color; }
                        final_color = lerp(final_color, color, behavior.mix());
                    },
                    None => continue
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

struct Camera {
    size: (f32, f32),
    lense_factor: (f32, f32),
    position: Vector3<f32>,
}

struct DirectionalLight {
    direction: Vector3<f32>,
    color: Vector3<f32>,
    intensity: f32,
}

impl DirectionalLight {
    fn diffuse_directional(&self, normal: Vector3<f32>, albedo: f32) -> Vector3<f32> {
        let light_power = InnerSpace::dot(normal, InnerSpace::normalize(self.direction)) * self.intensity;
        let light_reflected = albedo / std::f32::consts::PI;
        Vector3 {
            x: self.color.x * light_power * light_reflected,
            y: self.color.y * light_power * light_reflected,
            z: self.color.z * light_power * light_reflected,
        }
    }
}

struct World {
    entities: Vec<Box<dyn Entity>>,
    sun: DirectionalLight,
    sky: Material,
}

struct Sphere {
    position: Vector3<f32>,
    radius: f32,
    material: Material,
}

impl Sphere {
    fn new(position: Vector3<f32>, radius: f32, material: Material) -> Sphere {
        Sphere {
            position,
            radius,
            material,
        }
    }
}

impl Entity for Sphere {
    // fn collide(&self, ray: &Ray) -> ColliderResult {
    //     let l = self.position - ray.origin;
    //     let a = InnerSpace::dot(ray.direction, ray.direction);
    //     let b = 2.0 * InnerSpace::dot(ray.direction, l);
    //     let c = InnerSpace::dot(l, l) - self.radius;
    //     let s = solve_quadratic(a, b, c);
    //     if !s.0 { return ColliderResult::negative(); }
    //     if s.1 < 0.0 && s.2 < 0.0 { return ColliderResult::negative(); }
    //     if s.1 > s.2 { 
    //         let normal = InnerSpace::normalize(parameterize(ray, s.2) - self.position);
    //         return ColliderResult {
    //             collision: true,
    //             position: parameterize(ray, s.2),
    //             normal,
    //         };
    //     }
    //     let normal = InnerSpace::normalize(parameterize(ray, s.1) - self.position);
    //     ColliderResult {
    //         collision: true,
    //         position: parameterize(ray, s.1),
    //         normal,
    //     }
    // }

    fn collide(&self, ray: &Ray) -> ColliderResult { 
        let oc = ray.origin - self.position;
        let a = InnerSpace::dot(ray.direction, ray.direction);
        let b = 2.0 * InnerSpace::dot(oc, ray.direction);
        let c = InnerSpace::dot(oc, oc) - self.radius*self.radius;
        let discriminant = b*b - 4.0*a*c;
        if discriminant < 0.0 { return ColliderResult::negative() }
        let pos = ray.origin + (-b - discriminant.sqrt()) / (2.0*a) * ray.direction;
        ColliderResult {
            collision: true,
            position: pos,
            normal: InnerSpace::normalize(pos - self.position),
        }
    }

    fn material(&self) -> &Material {
        &self.material
    }
}

struct ColliderResult {
    collision: bool,
    position: Vector3<f32>,
    normal: Vector3<f32>,
}

impl ColliderResult {
    fn negative() -> ColliderResult {
        ColliderResult {
            collision: false,
            position: Vector3{x: 0.0, y: 0.0, z: 0.0},
            normal: Vector3{x: 0.0, y: 0.0, z: 0.0},
        }
    }
}

fn main() {
    println!("MAIN!");
    let raytracer = RayTracer::new_default_renderer((800, 450));
    let mut world = RayTracer::new_empty_world();

    let mat1 = Material::new_lambert_material(color_vec(104, 109, 118), 0.8, 0.0);
    let mat2 = Material::new_color_material(color_vec(55, 58, 64));

    let sphere = Sphere::new(Vector3{x: 0.0, y: 0.0, z: 50.0}, 2.0, mat1);
    let sphere2 = Sphere::new(Vector3{x: 2.0, y: 0.0, z:  -7.0}, 2.0, mat2);

    world.entities.push(Box::new(sphere));
    // world.entities.push(Box::new(sphere2));
    
    raytracer.render("./bruh.png".to_owned(), world);
}

fn reflect(d: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
    Vector3{
        x: n.y,
        y: n.y,
        z: n.y,
    }
}

fn cubemap(x: f32, y: f32, z: f32) -> (u32, f32, f32) {
  let absX = x.abs();
  let absY = y.abs();
  let absZ = z.abs();
  
  let isXPositive = x > 0.0;
  let isYPositive = y > 0.0;
  let isZPositive = z > 0.0;
  
  let mut maxAxis = 0 as f32;
  let mut uc = 0 as f32;
  let mut vc = 0 as f32;

  let mut index = 0 as u32;

  // POSITIVE X
  if isXPositive && absX >= absY && absX >= absZ {
    // u (0 to 1) goes from +z to -z
    // v (0 to 1) goes from -y to +y
    maxAxis = absX;
    uc = -z;
    vc = y;
    index = 0;
  }

  // NEGATIVE X
  if !isXPositive && absX >= absY && absX >= absZ {
    // u (0 to 1) goes from -z to +z
    // v (0 to 1) goes from -y to +y
    maxAxis = absX;
    uc = z;
    vc = y;
    index = 1;
  }

  // POSITIVE Y
  if isYPositive && absY >= absX && absY >= absZ {
    // u (0 to 1) goes from -x to +x
    // v (0 to 1) goes from +z to -z
    maxAxis = absY;
    uc = x;
    vc = -z;
    index = 2;
  }

  // NEGATIVE Y
  if !isYPositive && absY >= absX && absY >= absZ {
    // u (0 to 1) goes from -x to +x
    // v (0 to 1) goes from -z to +z
    maxAxis = absY;
    uc = x;
    vc = z;
    index = 3;
  }

  // POSITIVE Z
  if isZPositive && absZ >= absX && absZ >= absY {
    // u (0 to 1) goes from -x to +x
    // v (0 to 1) goes from -y to +y
    maxAxis = absZ;
    uc = x;
    vc = y;
    index = 4;
  }

  // NEGATIVE Z
  if !isZPositive && absZ >= absX && absZ >= absY {
    // u (0 to 1) goes from +x to -x
    // v (0 to 1) goes from -y to +y
    maxAxis = absZ;
    uc = -x;
    vc = y;
    index = 5;
  }

  // Convert range from -1 to 1 to 0 to 1
  let u = 0.5 * (uc / maxAxis + 1.0);
  let v = 0.5 * (vc / maxAxis + 1.0);

  (index, u, v)
}

fn solve_quadratic(a: f32, b: f32, c: f32) -> (bool, f32, f32) {
    let d = b * b - 4.0 * a * c;
    let mut x0: f32;
    let mut x1: f32;
    if d < 0.0 { 
        return (false, 0.0, 0.0); 
    } else if d == 0.0 { 
        x0 = - 0.5 * b / a;
        x1 = - 0.5 * b / a;
    } else {
        let q: f32;
        if b > 0.0 {
            q = -0.5 * (b + d.sqrt());
        } else {
            q = -0.5 * (b - d.sqrt());
        }
        x0 = q / a; 
        x1 = c / q;
    }
    if x0 > x1 {
        let tmp = x0;
        x0 = x1;
        x1 = tmp;
    }
    (true, x0, x1)
}

struct Ray {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    bounce: u32,
}

fn parameterize(ray: &Ray, t: f32) -> Vector3<f32> {
    ray.origin + ray.direction * t
}

fn vec_rgb(v: Vector3<f32>) -> image::Rgb<u8> {
    image::Rgb([(v.x * 255.0) as u8, (v.y * 255.0) as u8, (v.z * 255.0) as u8])
}

fn rgb_vec(i: image::Rgb<u8>) -> Vector3<f32> {
    color_vec(i[0], i[1], i[2])
}

fn color_vec(r: u8, g: u8, b: u8) -> Vector3<f32> {
    Vector3{x: (r as f32) / 255.0, y: (g as f32) / 255.0, z: (b as f32) / 255.0}
}

fn lerp(v1: Vector3<f32>, v2: Vector3<f32>, amount: f32) -> Vector3<f32> {
    Vector3{
        x: v1.x + (v2.x - v1.x) * amount,
        y: v1.y + (v2.y - v1.y) * amount,
        z: v1.z + (v2.z - v1.z) * amount,
    }
}
 
// Todo:
// - fix normal bug so that reflections work properly
// - create diffuse behavior & diffuse material (random ray shooting, little to no lambertian)
// - separate into multiple files
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