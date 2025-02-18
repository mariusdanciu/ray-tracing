use glam::{Vec2, Vec3};

use crate::{
    objects::{Intersection, Object3D},
    ray::{Ray, RayHit},
};

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub point: Vec3,
    pub max_dist: Option<Vec2>,
    pub material_index: usize,
}
impl Plane {
    pub fn new(normal: Vec3, point: Vec3, max_dist: Option<Vec2>, material_index: usize,) -> Object3D {
        Object3D::Plane(Plane {
            normal,
            point,
            max_dist,
            material_index,
        })
    }
}

impl Intersection for Plane {
    fn intersect(&self, ray: &Ray) -> Option<RayHit> {
        let denom = ray.direction.dot(self.normal);

        if denom.abs() < 1e-6 {
            return None;
        }

        let t = (self.point - ray.origin).dot(self.normal) / denom;

        if t < 0. {
            return None;
        }
        let hit_point = ray.origin + ray.direction * t;

        if let Some(Vec2 { x, y }) = self.max_dist {
            if hit_point.z.abs() > y || hit_point.x.abs() > x {
                return None;
            }
        }

        Some(RayHit {
            distance: t,
            point: hit_point,
            normal: self.normal,
            material_index: self.material_index,
            u: hit_point.x * 0.1,
            v: hit_point.z * 0.1,
        })
    }
}
