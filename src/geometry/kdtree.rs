extern crate cgmath;
use std::{cell::RefCell, ops::Deref, rc::Rc, sync::{Arc, Mutex}};

use crate::{common::{ColliderResult, Ray}, geometry::aabb::AABB };
use crate::common::Entity;
use cgmath::{InnerSpace, Point3, Vector3};

pub struct KDTree <T> {
    pub left: Option<Box<KDTree<T>>>,
    pub right: Option<Box<KDTree<T>>>,
    pub aa_bb: Option<AABB>,
    pub point: Option<f32>,
    pub leaf: Option<Vec<Arc<Mutex<T>>>>,
    pub axis: Option<usize>
}

impl <T> KDTree<T> {
    pub fn find_point(&self, point: Point3<f32>) -> Option<&KDTree<T>> {
        let mut axis = 0;
        if !self.aa_bb.as_ref().unwrap().contains(&point) { return None }
        let mut node = self;
        while let None = node.leaf {
            if point[axis] >= node.point.unwrap() { node = node.right.as_ref().unwrap(); }
            else { node = node.left.as_ref().unwrap(); }
            axis = (axis + 1) % 3;
        }
        Some(node)
    }

    pub fn translate_nodes(&mut self, vec: Vector3<f32>) {
        if let None = self.leaf {
            *self.point.as_mut().unwrap() += vec[self.axis.unwrap()];
            self.left.as_mut().unwrap().translate_nodes(vec);
            self.right.as_mut().unwrap().translate_nodes(vec);
        }
    }
}

impl <T: Entity> KDTree<T> {
    pub fn new(entities: Vec<Arc<Mutex<T>>>) -> Self {
        let bounding_box = AABB::from_entities(entities.iter().map(|a|a.deref().lock().unwrap()));
        return KDTree::build_tree(entities, 0, bounding_box);
    }

    fn build_tree(mut entities: Vec<Arc<Mutex<T>>>, depth: usize, bounding_box: AABB) -> Self {
        let axis = depth % 3;
        if entities.len() < 10 {
            return KDTree {
                left: None,
                right: None,
                point: None,
                axis: None,
                aa_bb: Some(bounding_box),
                leaf: Some(entities)
            }
        }
        let get_min_axis = |a: &T| a.bounding_box().min[axis];
        let get_max_axis = |a: &T| a.bounding_box().max[axis];

        let median_pos = entities.len() / 2;
        entities.sort_unstable_by(|a, b| get_min_axis(&a.lock().unwrap()).partial_cmp(&get_min_axis(&b.lock().unwrap())).unwrap());
        let partition = get_min_axis(&entities[median_pos].lock().unwrap());

        let entities_orig_length = entities.len();
        let mut right_half = entities.split_off(median_pos);
        let right_orig_length = right_half.len();

        for ent in &entities {
            if get_max_axis(&ent.lock().unwrap()) >= partition {
                right_half.push(ent.clone());
            }
        }

        for ent in &right_half[..right_orig_length] {
            if get_min_axis(&ent.lock().unwrap()) < partition {
                entities.push(ent.clone());
            }
        }

        if entities.len() >= entities_orig_length {
            return KDTree {
                left: None,
                right: None,
                point: None,
                axis: None,
                aa_bb: Some(bounding_box),
                leaf: Some(entities)
            }
        } else if right_half.len() >= entities_orig_length {
            return KDTree {
                left: None,
                right: None,
                point: None,
                axis: None,
                aa_bb: Some(bounding_box),
                leaf: Some(right_half)
            }
        }
        

        let mut left_bb_max = bounding_box.max;
        left_bb_max[axis] = partition;
        let left_bb = AABB {
            min: bounding_box.min,
            max: left_bb_max
        };
        let mut right_bb_min = bounding_box.min;
        right_bb_min[axis] = partition;
        let right_bb = AABB {
            min: right_bb_min,
            max: bounding_box.max
        };

        return KDTree {
            aa_bb: Some(bounding_box),
            left: Some(Box::new(KDTree::build_tree(entities, depth + 1, left_bb))),
            right: Some(Box::new(KDTree::build_tree(right_half, depth + 1, right_bb))),
            axis: Some(axis),
            point: Some(partition),
            leaf: None
        }
    }

    pub fn collide(&self, ray: &Ray) -> ColliderResult {
        let mut collision = self.aa_bb.as_ref().unwrap().collide(ray);
        if !collision.collision {
            return ColliderResult::negative();
        }
        let mut point = collision.position + ray.direction * 0.01;
        loop {
            let leaf = self.find_point(point);
            if let Some(node) = leaf {
                let mut min_distance = std::f32::MAX;
                let mut closest: Option<ColliderResult> = None;
                for entity in node.leaf.as_ref().unwrap() {
                    collision = entity.lock().unwrap().collide(ray);
                    if collision.collision {
                        let distance = (collision.position - ray.origin).magnitude2();
                        if distance < min_distance {
                            min_distance = distance;
                            closest = Some(collision);
                        }
                    }
                }
                if let Some(col) = closest {
                    return col;
                }
                let new_ray = Ray {
                    origin: point,
                    direction: ray.direction,
                    bounce: 0
                };
                collision = node.aa_bb.as_ref().unwrap().collide(&new_ray);
                point = collision.position + ray.direction * 0.01;
            } else {
                return ColliderResult::negative();
            }
        }
    }
}

// If we want to have objects of multiple types in the tree, we need a boxed variant.

// ----- ALMOST IDENTICAL DUPLICATED CODE TO THE ABOVE -------------------------------

impl <T: Entity + ?Sized> KDTree<Box<T>> {
    pub fn new_boxed(entities: Vec<Arc<Mutex<Box<T>>>>) -> Self {
        let bounding_box = AABB::from_dyn_entities(&entities);
        return KDTree::build_tree_boxed(entities, 0, bounding_box);
    }

    fn build_tree_boxed(mut entities: Vec<Arc<Mutex<Box<T>>>>, depth: usize, bounding_box: AABB) -> Self {
        let axis = depth % 3;
        if entities.len() < 10 {
            return KDTree {
                left: None,
                right: None,
                point: None,
                axis: None,
                aa_bb: Some(bounding_box),
                leaf: Some(entities)
            }
        }
        let get_min_axis = |a: &T| a.bounding_box().min[axis];
        let get_max_axis = |a: &T| a.bounding_box().max[axis];

        let median_pos = entities.len() / 2;
        entities.sort_unstable_by(|a, b| get_min_axis(&a.lock().unwrap()).partial_cmp(&get_min_axis(&b.lock().unwrap())).unwrap());
        let partition = get_min_axis(&entities[median_pos].lock().unwrap());

        let entities_orig_length = entities.len();
        let mut right_half = entities.split_off(median_pos);
        let right_orig_length = right_half.len();

        for ent in &entities {
            if get_max_axis(&ent.lock().unwrap()) >= partition {
                right_half.push(ent.clone());
            }
        }

        for ent in &right_half[..right_orig_length] {
            if get_min_axis(&ent.lock().unwrap()) < partition {
                entities.push(ent.clone());
            }
        }

        if entities.len() >= entities_orig_length {
            return KDTree {
                left: None,
                right: None,
                point: None,
                axis: None,
                aa_bb: Some(bounding_box),
                leaf: Some(entities)
            }
        } else if right_half.len() >= entities_orig_length {
            return KDTree {
                left: None,
                right: None,
                point: None,
                axis: None,
                aa_bb: Some(bounding_box),
                leaf: Some(right_half)
            }
        }
        

        let mut left_bb_max = bounding_box.max;
        left_bb_max[axis] = partition;
        let left_bb = AABB {
            min: bounding_box.min,
            max: left_bb_max
        };
        let mut right_bb_min = bounding_box.min;
        right_bb_min[axis] = partition;
        let right_bb = AABB {
            min: right_bb_min,
            max: bounding_box.max
        };

        return KDTree {
            aa_bb: Some(bounding_box),
            left: Some(Box::new(KDTree::build_tree_boxed(entities, depth + 1, left_bb))),
            right: Some(Box::new(KDTree::build_tree_boxed(right_half, depth + 1, right_bb))),
            axis: Some(axis),
            point: Some(partition),
            leaf: None
        }
    }

    pub fn collide_boxed(&self, ray: &Ray) -> ColliderResult {
        let mut collision = self.aa_bb.as_ref().unwrap().collide(ray);
        if !collision.collision {
            return ColliderResult::negative();
        }
        let mut point = collision.position + ray.direction * 0.01;
        loop {
            let leaf = self.find_point(point);
            if let Some(node) = leaf {
                let mut min_distance = std::f32::MAX;
                let mut closest: Option<ColliderResult> = None;
                for entity in node.leaf.as_ref().unwrap() {
                    collision = entity.lock().unwrap().collide(ray);
                    if collision.collision {
                        let distance = (collision.position - ray.origin).magnitude2();
                        if distance < min_distance {
                            min_distance = distance;
                            closest = Some(collision);
                        }
                    }
                }
                if let Some(col) = closest {
                    return col;
                }
                let new_ray = Ray {
                    origin: point,
                    direction: ray.direction,
                    bounce: 0
                };
                collision = node.aa_bb.as_ref().unwrap().collide(&new_ray);
                point = collision.position + ray.direction * 0.01;
            } else {
                return ColliderResult::negative();
            }
        }
    }
}