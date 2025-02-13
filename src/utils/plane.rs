use glam::{Vec2, Vec3};

use crate::ray::{Ray, RayHit};


pub fn plane_intersection(
    ray: &Ray,
    normal: Vec3,
    p: Vec3,
    max_dist: Option<Vec2>,
    material_index: usize,
) -> Option<RayHit> {
    let denom = ray.direction.dot(normal);

    if denom.abs() < 1e-6 {
        return None;
    }

    let t = (p - ray.origin).dot(normal) / denom;

    if t < 0. {
        return None;
    }
    let hit_point = ray.origin + ray.direction * t;

    if let Some(Vec2 { x, y }) = max_dist {
        if hit_point.z.abs() > y || hit_point.x.abs() > x {
            return None;
        }
    }

    Some(RayHit {
        distance: t,
        point: hit_point,
        normal,
        material_index,
        u: hit_point.x * 0.1,
        v: hit_point.z * 0.1,
    })
}
