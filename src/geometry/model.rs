extern crate cgmath;

use crate::material::Material;
use crate::common::{Entity, ColliderResult, Ray};
use crate::geometry::triangle::Triangle;
use crate::geometry::kdtree::KDTree;
use crate::geometry::aabb::AABB;

use cgmath::{Vector3, InnerSpace, Point3, EuclideanSpace};
use obj::{load_obj, Obj}; 
use std::fs::File;
use std::io::BufReader;
use crate::cgmath::Transform;

pub struct Model {
    material: Material,
    triangles: Vec<Triangle>,
    tree: KDTree<Triangle>,
    position: Point3<f32>
}

impl Model {
    pub fn new(path: &str, material: Material, position: Point3<f32>, scale: Vector3<f32>) -> Model {
        println!("Opening model @ {}", path);
        let mut input = BufReader::new(File::open(path).unwrap());
        let input = BufReader::new(File::open(path).unwrap());
        let model: Obj = load_obj(input).unwrap();
        let mut triangles = Vec::new();
        let translation = position.to_vec();
        let transform = cgmath::Matrix4::from_translation(translation) * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
        for i in (0..model.indices.len()-4).step_by(3) {
            let index = model.indices[i] as usize;
            let v0 = vertex2point(model.vertices[model.indices[i] as usize]);
            let v1 = vertex2point(model.vertices[model.indices[i+1] as usize]);
            let v2 = vertex2point(model.vertices[model.indices[i+2] as usize]);
            let n0 = vertex2normal(model.vertices[model.indices[i] as usize]);
            let n1 = vertex2normal(model.vertices[model.indices[i+1] as usize]);
            let n2 = vertex2normal(model.vertices[model.indices[i+2] as usize]);
            triangles.push(Triangle::new(
                transform.transform_point(v0), 
                transform.transform_point(v1), 
                transform.transform_point(v2), 
            (n0+n1+n2).normalize()));
        }
        println!("Model has {} triangles.", triangles.len());
        println!("Building k-d tree with model's triangles...");
        let tree = KDTree::new(triangles.clone());
        Model {
            material,
            triangles: triangles,
            position,
            tree
        }
    }

    pub fn compute_normals(&mut self) {
        for t in &mut self.triangles {
            t.compute_normal();
        }
    }
}

impl Entity for Model {
    fn collide(&self, ray: &Ray) -> ColliderResult {
        self.tree.collide(ray)
    }

    fn material(&self) -> Option<&Material> {
        Some(&self.material)
    }

    fn bounding_box(&self) -> AABB {
        return AABB::from_entities(&self.triangles);
    }

    fn position(&self) -> Point3<f32> {
        Point3 {
            x: self.position.x,
            y: self.position.y,
            z: self.position.z
        }
    }
}

fn vertex2point(v: obj::Vertex) -> Point3<f32> {
    Point3 {
        x: v.position[0],
        y: v.position[1],
        z: v.position[2],
    }
}

fn vertex2normal(v: obj::Vertex) -> Vector3<f32> {
    Vector3 {
        x: v.normal[0],
        y: v.normal[1],
        z: v.normal[2],
    }.normalize()
}

struct Scene {
    models: Vec<Model>,
    position: Point3<f32>,
    tree: KDTree<Model>
}

// impl Entity for Scene {
    
// }