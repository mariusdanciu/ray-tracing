use std::f32;

use glam::{vec2, Vec2, Vec3};

pub fn box_sdf(p: Vec3, dimension: Vec3) -> f32 {
    let corner_radius = 0.1;
    let q = p.abs() - dimension + corner_radius;

    q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0) - corner_radius
}

pub fn sphere_sdf(p: Vec3, radius: f32) -> f32 {
    p.length() - radius
}

pub fn plane_sdf(p: Vec3, plane_point: Vec3, normal: Vec3) -> f32 {
    (p - plane_point).dot(normal)
}

pub fn cylinder_sdf(p: Vec3, radius: f32, corner_radius: f32, height: f32) -> f32 {
    let d = vec2(vec2(p.x, p.z).length(), (p.y).abs()) - vec2(radius, height * 0.5) + corner_radius;
    let dist = (d.max(Vec2::ZERO)).length() + d.x.max(d.y).min(0.0) - corner_radius;

    dist
}
