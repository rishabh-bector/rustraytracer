extern crate cgmath;

use crate::material::Material;
use crate::common::{Entity, ColliderResult, Ray};
use crate::geometry::triangle::Triangle;

use cgmath::{Vector3, InnerSpace};
use obj::{load_obj, Obj}; 
use std::fs::File;
use std::io::BufReader;

pub struct Model {
    material: Material,
    triangles: Vec<Triangle>,
}

impl Model {
    pub fn new(path: &str, material: Material, transform: cgmath::Matrix4<f32>) -> Model {
        println!("Opening model @ {}", path);
        let mut input = BufReader::new(File::open(path).unwrap());
        let input = BufReader::new(File::open(path).unwrap());
        let model: Obj = load_obj(input).unwrap();
        let mut triangles = Vec::new();
        for i in (0..model.indices.len()-4).step_by(3) {
            let index = model.indices[i] as usize;
            let v0 = vertex2vec(model.vertices[model.indices[i] as usize]);
            let v1 = vertex2vec(model.vertices[model.indices[i+1] as usize]);
            let v2 = vertex2vec(model.vertices[model.indices[i+2] as usize]);
            let n0 = vertex2normal(model.vertices[model.indices[i] as usize]);
            let n1 = vertex2normal(model.vertices[model.indices[i+1] as usize]);
            let n2 = vertex2normal(model.vertices[model.indices[i+2] as usize]);
            triangles.push(Triangle::new(v0, v1, v2, (n0+n1+n2).normalize()));
        }
        println!("Model has {} triangles.", triangles.len());
        Model {
            material,
            triangles,
        }
    }
}

impl Entity for Model {
    fn collide(&self, ray: &Ray) -> ColliderResult {
        let mut mindist = f32::MAX;
        let mut minresult = ColliderResult::negative();
        for t in self.triangles.iter() {
            let col = t.collide(ray);
            let distance = (col.position - ray.origin).magnitude();
            if col.collision && distance < mindist {
                mindist = distance;
                minresult = col;
            }
        }
        minresult
    }

    fn material(&self) -> Option<&Material> {
        Some(&self.material)
    }
}

fn vertex2vec(v: obj::Vertex) -> Vector3<f32> {
    Vector3 {
        x: v.position[0],
        y: v.position[1],
        z: v.position[2]+20.0,
    }
}

fn vertex2normal(v: obj::Vertex) -> Vector3<f32> {
    Vector3 {
        x: v.normal[0],
        y: v.normal[1],
        z: v.normal[2],
    }.normalize()
}