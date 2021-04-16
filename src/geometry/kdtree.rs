extern crate cgmath;
use std::{sync::{Arc}};

use crate::{common::{ColliderResult, Ray}, geometry::aabb::AABB };
use crate::common::Entity;
use cgmath::{InnerSpace, Point3, Vector3};

struct RopeType<T>([Option<*const KDTree<T>>; 6]);
unsafe impl <T> Sync for RopeType<T>{}
unsafe impl <T> Send for RopeType<T>{}

pub struct KDTree <T> {
    aa_bb: AABB,
    left: Option<Box<KDTree<T>>>,
    right: Option<Box<KDTree<T>>>,
    partition: Option<f64>,
    leaf: Option<Vec<Arc<T>>>,
    ropes: RopeType<T>,
    axis: Option<usize>
}

impl <T> KDTree<T> {
    pub fn find_point(&self, point: Point3<f64>) -> Option<&KDTree<T>> {
        if !self.aa_bb.contains(&point) { return None }

        let mut node = self;
        while let None = node.leaf {
            let axis = node.axis.unwrap();
            node = if point[axis] >= *node.partition.as_ref().unwrap() { node.right.as_ref().unwrap() }
            else { node.left.as_ref().unwrap() }
        }
        Some(node)
    }

    pub fn translate_nodes(&self, vec: Vector3<f64>) {
        if let None = self.leaf {
            let partition_mut = self.partition.as_ref().unwrap() as *const f64 as *mut f64;
            unsafe {*partition_mut += vec[self.axis.unwrap()]};
            self.left.as_ref().unwrap().translate_nodes(vec);
            self.right.as_ref().unwrap().translate_nodes(vec);
        }
    }
}

impl <T: Entity> KDTree<T> {
    pub fn new(entities: Vec<Arc<T>>) -> Self {
        let bounding_box = AABB::from_entities(entities.iter().map(|a|a.as_ref()));
        let mut tree = KDTree::build_tree(entities, 0, bounding_box);
        tree.make_ropes(RopeType([None, None, None, None, None, None]), tree.as_ref());
        *tree
    }

    fn build_tree(mut entities: Vec<Arc<T>>, depth: usize, bounding_box: AABB) -> Box<Self> {
        let axis = depth % 3;
        if entities.len() < 5 {
            return Box::new(KDTree {
                left: None,
                right: None,
                partition: None,
                axis: None,
                aa_bb: bounding_box,
                ropes: RopeType([None, None, None, None, None, None]),
                leaf: Some(entities)
            });
        }
        let get_min_axis = |a: &T| a.bounding_box().min[axis];
        let get_max_axis = |a: &T| a.bounding_box().max[axis];

        let median_pos = entities.len() / 2;
        entities.sort_unstable_by(|a, b| get_min_axis(&a).partial_cmp(&get_min_axis(&b)).unwrap());
        let mut partition = get_min_axis(&entities[median_pos]);
        
        if partition == bounding_box.min[axis] || partition == bounding_box.max[axis] {
            partition = (bounding_box.min[axis] + bounding_box.max[axis]) / 2.;
        }

        let entities_orig_length = entities.len();
        let mut right_half = entities.split_off(median_pos);
        let right_orig_length = right_half.len();

        for ent in &entities {
            if get_max_axis(&ent) >= partition {
                right_half.push(ent.clone());
            }
        }

        for ent in &right_half[..right_orig_length] {
            if get_min_axis(&ent) < partition {
                entities.push(ent.clone());
            }
        }

        if entities.len() >= entities_orig_length {
            return Box::new(KDTree {
                left: None,
                right: None,
                partition: None,
                axis: None,
                aa_bb: bounding_box,
                ropes: RopeType([None, None, None, None, None, None]),
                leaf: Some(entities)
            });
        } else if right_half.len() >= entities_orig_length {
            return Box::new(KDTree {
                left: None,
                right: None,
                partition: None,
                axis: None,
                ropes: RopeType([None, None, None, None, None, None]),
                aa_bb: bounding_box,
                leaf: Some(right_half)
            });
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

        Box::new(KDTree {
            aa_bb: bounding_box,
            left: Some(KDTree::build_tree(entities, depth + 1, left_bb)),
            right: Some(KDTree::build_tree(right_half, depth + 1, right_bb)),
            axis: Some(axis),
            partition: Some(partition),
            ropes: RopeType([None, None, None, None, None, None]),
            leaf: None
        })
    }
    
    fn make_ropes(&mut self, ropes: RopeType<T>, root: *const KDTree<T>) {
        let root = unsafe {&*root};
        let mut ropes = ropes.0;
        if let None = self.leaf {
            let axis = self.axis.unwrap();

            for i in 0..6 {
                let mut node;
                if let Some(n) = ropes[i] {
                    node = unsafe {&*n};
                    while let None = node.leaf {
                        if node.axis.unwrap() == i % 3 {
                            node = if i < 3 { node.right.as_ref().unwrap() }
                            else { node.left.as_ref().unwrap() }
                        } else if node.partition.unwrap() > self.aa_bb.max[node.axis.unwrap()] {
                            node = node.left.as_ref().unwrap();
                        } else if node.partition.unwrap() < self.aa_bb.min[node.axis.unwrap()] {
                            node = node.right.as_ref().unwrap();
                        } else { break }
                    }
                    ropes[i] = Some(node);
                }
            }

            let left = self.left.as_mut().unwrap();
            let right = self.right.as_mut().unwrap();

            let mut left_ropes = ropes;
            left_ropes[axis + 3] = Some(right.as_ref());
            left.make_ropes(RopeType(left_ropes), root);

            let mut right_ropes = ropes;
            right_ropes[axis] = Some(left.as_ref());
            right.make_ropes(RopeType(right_ropes), root);
        } else {
            self.ropes = RopeType(ropes);
            for i in 0..6 {
                if let None = ropes[i] {
                    if i < 3 {
                        if self.aa_bb.min[i % 3] != root.aa_bb.min[i % 3] {
                            panic!()
                        }
                    } else {
                        if self.aa_bb.max[i % 3] != root.aa_bb.max[i % 3] {
                            panic!()
                        }
                    }
                }
            }
        }
    }

    pub fn collide(&self, ray: &Ray) -> ColliderResult {
        let mut collision = self.aa_bb.collide(ray);
        if !collision.collision {
            return ColliderResult::negative();
        }
        let mut point = collision.position + ray.direction * 0.01;
        let mut next_leaf:Option<&KDTree<T>> = self.find_point(point);
        loop {
            if let Some(node) = next_leaf {
                let mut min_distance = std::f64::MAX;
                let mut closest: Option<ColliderResult> = None;
                for entity in node.leaf.as_ref().unwrap() {
                    collision = entity.collide(ray);
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
                collision = node.aa_bb.collide(&new_ray);
                point = collision.position + ray.direction * 0.001;
                let i = get_plane(point, node.aa_bb);
                if let Some(neighbor) = node.ropes.0[i as usize].as_ref() {
                    let neighbor = unsafe {&**neighbor};
                    next_leaf = neighbor.find_point(point);
                } else {
                    return ColliderResult::negative();
                }
            } else {
                return ColliderResult::negative();
            }
        }
    }
}

fn get_plane(point: Point3<f64>, aa_bb: AABB) -> u8 {
    return if point.x < aa_bb.min.x { 0 }
            else if point.y < aa_bb.min.y { 1 }
            else if point.z < aa_bb.min.z { 2 }
            else if point.x > aa_bb.max.x { 3 }
            else if point.y > aa_bb.max.y { 4 }
            else { 5 };
}