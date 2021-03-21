extern crate cgmath;

use crate::material::Material;
use crate::common::{Entity, ColliderResult, Ray};
use crate::geometry::triangle::Triangle;
use crate::geometry::kdtree::KDTree;
use crate::geometry::aabb::AABB;

use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Point3, Vector3};
use obj::{load_obj, Obj}; 
use std::{fs::File, ops::Deref, sync::{Arc, Mutex}};
use std::io::BufReader;
use crate::cgmath::Transform;

pub struct Model {
    material: Material,
    tree: KDTree<Triangle>,
    position: Point3<f32>,
    triangles: Vec<Arc<Mutex<Triangle>>>,
    aa_bb: AABB
}

impl Model {
    pub fn new(path: &str, material: Material, position: Point3<f32>, scale: Vector3<f32>) -> Model {
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
        let triangles: Vec<Arc<Mutex<Triangle>>> = triangles.into_iter().map(|a| Arc::new(Mutex::new(a))).collect();
        Model {
            material,
            aa_bb: AABB::from_entities(triangles.iter().map(|a|a.deref().lock().unwrap())),
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

    fn position(&self) -> Point3<f32> {
        Point3 {
            x: self.position.x,
            y: self.position.y,
            z: self.position.z
        }
    }

    fn translate(&mut self, vec: Vector3<f32>) {
        for triangle in &self.triangles {
            triangle.lock().unwrap().translate(vec);
        }
        self.tree.translate_nodes(vec);
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

pub struct Scene {
    tree: KDTree<Box<dyn Entity>>,
    models: Vec<Arc<Mutex<Box<dyn Entity>>>>,
    position: Point3<f32>,
    aa_bb: AABB
}

impl Scene {
    pub fn new(models: Vec<Box<dyn Entity>>, position: Point3<f32>) -> Self {
        let models: Vec<Arc<Mutex<Box<dyn Entity>>>> = models.into_iter().map(|a| Arc::new(Mutex::new(a))).collect();
        Scene {
            aa_bb: AABB::from_dyn_entities(&models),
            tree: KDTree::new_boxed(models.clone()),
            models,
            position
        }
    }
}

impl Entity for Scene {
    fn collide(&self, ray: &Ray) -> ColliderResult {
        self.tree.collide_boxed(ray)
    }

    fn material(&self) -> Option<&Material> {
        None
    }

    fn bounding_box(&self) -> AABB {
        self.aa_bb.clone()
    }

    fn position(&self) -> Point3<f32> {
        self.position
    }

    fn translate(&mut self, vec: Vector3<f32>) {
        for model in &self.models {
            model.lock().unwrap().translate(vec);
        }
        self.tree.translate_nodes(vec);
    }
}