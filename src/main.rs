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

struct LambertBehavior {
    albedo: f32,
    mix: f32,
    color: Vector3<f32>
}

impl LambertBehavior {
    fn new(albedo: f32, mix: f32, color: Vector3<f32>) -> LambertBehavior {
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
        let mut result = Vector3 {x: 0., y: 0., z: 0.};
        for light_source in world.light_sources.iter() {
            if light_source.visible(collision.position, collision.normal) {
                let LightRay { power, direction } = light_source.illuminate(collision.position, collision.normal);
                let power = power * (self.albedo / std::f32::consts::PI) * -collision.normal.dot(direction);
                let power = power.max(0.);
                result += self.color * power;
            }
        }
        Some( result )
    }

    fn mix(&self) -> f32 { self.mix }
}

struct PhongBehavior {
    mix: f32,
    alpha: i32
}

impl RayBehavior for PhongBehavior {
    fn mix(&self) -> f32 { self.mix }
    fn compute (
        &self, 
        ray: &Ray,
        world: &World,
        collision: &ColliderResult,
        _tracer: &RayTracer,
    ) -> Option<Vector3<f32>> {
        let mut result = Vector3 {x: 0., y: 0., z: 0.};
        for light_source in world.light_sources.iter() {
            if light_source.visible(collision.position, collision.normal) {
                let LightRay { power, direction } = light_source.illuminate(collision.position, collision.normal);
                let ray_bisector = (-direction - ray.direction).normalize();
                let power = power * ray_bisector.dot(collision.normal).max(0.).powi(self.alpha);
                result += light_source.color() * power;
            }
        }
        Some( result )
    }
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
        if ray.bounce > 3 { return None };
        let reflected = Ray{
            origin: collision.position + collision.normal * 0.3,
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
    pub shaders: Vec<Box<dyn RayBehavior>>,
    pub color: Vector3<f32>
}

impl Material {

    fn new_lambert_material(color: Vector3<f32>, albedo: f32, lambert: f32, reflective: f32, phong: f32) -> Material {
        let lambert_behavior = LambertBehavior::new(albedo, lambert, color);
        let ref_be = ReflectionBehavior::new(reflective);
        let phong_behavior = PhongBehavior { alpha: 60, mix: phong };
        let mut shaders: Vec<Box<dyn RayBehavior>> = Vec::new();
        shaders.push(Box::new(lambert_behavior));
        shaders.push(Box::new(ref_be));
        shaders.push(Box::new(phong_behavior));
        Material { shaders, color }
    }

    fn new_sky_material(cubemap_folder: &str) -> Material {
        let cubemap_behavior = CubemapBehavior::new(cubemap_folder, 1.0);
        let mut shaders: Vec<Box<dyn RayBehavior>> = Vec::new();
        shaders.push(Box::new(cubemap_behavior));
        Material { shaders, color: color_vec(0, 0, 0) }
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

    fn new_empty_world(skybox: &str) -> World {
        let entities: Vec<Box<dyn Entity>> = Vec::new();
        let sun = DirectionalLight::new(
            Vector3{x: 1.0, y: -0.5, z: 1.0},
            color_vec(230, 230, 230),
            2.0
        );

        let light_sources: Vec<Box<dyn LightSource>> = vec![Box::new(sun)];

        let sky = Material::new_sky_material(skybox);

        World {entities, light_sources, sky, ambient: 0.15}
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

            // if x == 0 && y == 0 {
            //     println!("{:?}", ray.direction);
            // }
        }

        match img.save(output) {
            Ok(_) => println!("Saved!"),
            Err(e) => println!("{}", e), 
        };

        let duration = timer.elapsed();
        println!("Finished in {}ms", duration.as_millis());
    }
    
    fn cast(&self, ray: &Ray, world: &World) -> Vector3<f32> {
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
            let material = ent.material();

            let mut final_color: Vector3<f32> = material.color * world.ambient;
            for behavior in material.shaders.iter() {
                match behavior.as_ref().compute(ray, world, &result, self) {
                    Some(color) => {
                        final_color = final_color + color * behavior.mix();
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

struct LightRay {
    power: f32,
    direction: Vector3<f32>
}

trait LightSource {
    fn illuminate(&self, pos: Vector3<f32>, normal: Vector3<f32>) -> LightRay;
    fn visible(&self, pos: Vector3<f32>, normal: Vector3<f32>) -> bool;
    fn color(&self) -> Vector3<f32>;
}

struct DirectionalLight {
    direction: Vector3<f32>,
    color: Vector3<f32>,
    intensity: f32,
}

impl DirectionalLight {
    fn new(direction: Vector3<f32>, color: Vector3<f32>, intensity: f32) -> Self {
        let direction = direction.normalize();
        Self {
            direction,
            color,
            intensity
        }
    }
}

impl LightSource for DirectionalLight {
    fn illuminate(&self, _pos: Vector3<f32>, normal: Vector3<f32>) -> LightRay {
        LightRay { power: self.intensity, direction: self.direction }
    }

    fn visible(&self, _pos: Vector3<f32>, normal: Vector3<f32>) -> bool { normal.dot(self.direction) < 0. }

    fn color(&self) -> Vector3<f32> { self.color }
}

struct PointLight {
    position: Vector3<f32>,
    color: Vector3<f32>,
    brightness: f32,
}

struct World {
    entities: Vec<Box<dyn Entity>>,
    light_sources: Vec<Box<dyn LightSource>>,
    sky: Material,
    ambient: f32
}

struct Sphere {
    position: Vector3<f32>,
    radius: f32,
    radius2: f32,
    material: Material,
}

impl Sphere {
    fn new(position: Vector3<f32>, radius: f32, material: Material) -> Sphere {
        Sphere {
            position,
            radius,
            radius2: radius.powi(2),
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
        let l = self.position - ray.origin;
        let tca = l.dot(ray.direction);
        if tca < 0.0 { return ColliderResult::negative() }
        let d2 = l.magnitude2() - tca.powi(2);
        if d2 > self.radius2 { return ColliderResult::negative() }
        let thc = (self.radius2 - d2).sqrt();
        let pos = ray.origin + (tca - thc) * ray.direction;

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
    let raytracer = RayTracer::new_default_renderer((1920, 1080));
    let mut world = RayTracer::new_empty_world("./cubemaps/hd_blue_sunset");

    let mat1 = Material::new_lambert_material(color_vec(100, 100, 200), 0.8, 1.0, 0.2, 0.3);
    let mat2 = Material::new_lambert_material(color_vec(0, 0, 0), 0.8, 0.0, 1.0, 0.0);

    let sphere = Sphere::new(Vector3{x: 0.0, y: 0.0, z: 10.0}, 2.0, mat1);
    let sphere2 = Sphere::new(Vector3{x: 2.0, y: 0.0, z:  5.0}, 1.0, mat2);

    world.entities.push(Box::new(sphere));
    world.entities.push(Box::new(sphere2));
    
    raytracer.render("./bruh.png".to_owned(), world);
}

fn reflect(d: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
    d - (n * (n.dot(d)) * 2.0)
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