extern crate cgmath;

use cgmath::{Point3, Vector3};
use crate::common::*;
use crate::geometry::{kdtree::KDTree, aabb::AABB};
use crate::material::Material;
use std::rc::Rc;

// Invocation: entity_enum! (Name, Type1, Type2, ...)
// Creates an enum type with name Name which auto-implements entity.
// Instantiate each enum value like object1 = Type1(Value1).
#[macro_export]
macro_rules! entity_enum {
    ($name:ident, $($x: ident),+) => {
        use derive_entity::*;
        #[derive(Entity)]
        enum $name {
            $($x($x),)+
        }
    };
}

// Invocation: scene! (EnumType, position, ObjType1(obj1), ObjType2(obj2), ...)
// Creates a scene from the given objects using an enum type created with entity_enum!
#[macro_export]
macro_rules! scene {
    ($typeName: ident, $position: expr, $($x:ident($y: ident)),+) => {
        Scene::new(vec![$($typeName::$x($y)),+], $position)
    };
}

pub struct Scene<T: Entity> {
    tree: KDTree<T>,
    models: Vec<Rc<T>>,
    position: Point3<f64>,
    aa_bb: AABB
}

impl <T: Entity> Scene<T> {
    pub fn new(models: Vec<T>, position: Point3<f64>) -> Self {
        let models: Vec<Rc<T>> = models.into_iter().map(|a| Rc::new(a)).collect();
        Scene {
            aa_bb: AABB::from_entities(models.iter().map(|a|a.as_ref())),
            tree: KDTree::new(models.clone()),
            models,
            position
        }
    }
}

impl <T: Entity> Entity for Scene<T> {
    fn collide(&self, ray: &Ray) -> ColliderResult {
        self.tree.collide(ray)
    }

    fn material(&self) -> Option<&Material> {
        None
    }

    fn bounding_box(&self) -> AABB {
        self.aa_bb.clone()
    }

    fn position(&self) -> Point3<f64> {
        self.position
    }

    fn translate(&mut self, vec: Vector3<f64>) {
        for model in &self.models {
            let model_mut = model.as_ref() as *const T as *mut T;
            unsafe {(*model_mut).translate(vec)}
        }
        self.tree.translate_nodes(vec);
    }
}