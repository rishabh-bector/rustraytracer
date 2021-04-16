extern crate cgmath;

use crate::material::Material;
use crate::common::{Entity, ColliderResult, Ray};
use crate::geometry::triangle::Triangle;
use crate::geometry::kdtree::KDTree;
use crate::geometry::aabb::AABB;

use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Point3, Vector3};
use obj::{load_obj, Obj}; 
use std::{fs::File, sync::{Arc}};
use std::io::BufReader;
use crate::cgmath::Transform;

pub struct Model {
    material: Material,
    tree: KDTree<Triangle>,
    position: Point3<f64>,
    triangles: Vec<Arc<Triangle>>,
    aa_bb: AABB
}

impl Model {
    pub fn new(path: &str, material: Material, position: Point3<f64>, scale: Vector3<f64>) -> Model {
        println!("Opening model @ {}", path);
        let input = BufReader::new(File::open(path).unwrap());
        let model: Obj = load_obj(input).unwrap();
        let mut triangles = Vec::new();
        let translation = position.to_vec();
        let transform = Matrix4::from_translation(translation) * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
        for i in (0..model.indices.len()-4).step_by(3) {
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
                (n0 + n1 + n2).normalize(),
                material.clone()));
            
        }
        println!("Model has {} triangles.", triangles.len());
        println!("Building k-d tree with model's triangles...");
        let triangles: Vec<Arc<Triangle>> = triangles.into_iter().map(|a| Arc::new(a)).collect();
        Model {
            material,
            aa_bb: AABB::from_entities(triangles.iter().map(|a|a.as_ref())),
            position,
            tree: KDTree::new(triangles.clone()),
            triangles
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
        self.aa_bb.clone()
    }

    fn position(&self) -> Point3<f64> {
        Point3 {
            x: self.position.x,
            y: self.position.y,
            z: self.position.z
        }
    }

    fn translate(&mut self, vec: Vector3<f64>) {
        for triangle in &self.triangles {
            let triangle = Arc::as_ptr(triangle) as *mut Triangle;
            unsafe { (*triangle).translate(vec); }
        }
        self.tree.translate_nodes(vec);
    }
}

fn vertex2point(v: obj::Vertex) -> Point3<f64> {
    Point3 {
        x: v.position[0] as f64,
        y: v.position[1] as f64,
        z: v.position[2] as f64,
    }
}

fn vertex2normal(v: obj::Vertex) -> Vector3<f64> {
    Vector3 {
        x: v.normal[0] as f64,
        y: v.normal[1] as f64,
        z: v.normal[2] as f64,
    }.normalize()
}