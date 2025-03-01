use glam::{vec3, Mat4, Vec3, Vec3Swizzles};

use crate::objects::Texture;

pub static DEGREES: f32 = std::f32::consts::PI / 180.;

pub fn reflect(vec: Vec3, normal: Vec3) -> Vec3 {
    vec - (2. * (vec.dot(normal))) * normal
}

pub fn rotate_y_mat(o: f32) -> Mat4 {
    let (s, c) = f32::sin_cos(o);
    Mat4::from_cols_array(&[c, 0., s, 0., 0., 1., 0., 0., -s, 0., c, 0., 0., 0., 0., 1.])
}

pub fn rotate_x_mat(o: f32) -> Mat4 {
    let (s, c) = f32::sin_cos(o);
    Mat4::from_cols_array(&[1., 0., 0., 0., 0., c, -s, 0., 0., s, c, 0., 0., 0., 0., 1.])
}

pub fn step(a: Vec3, b: Vec3) -> Vec3 {
    let x = if b.x < a.x { 0.0 } else { 1.0 };
    let y = if b.y < a.y { 0.0 } else { 1.0 };
    let z = if b.z < a.z { 0.0 } else { 1.0 };

    return vec3(x, y, z);
}

pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

pub fn mix(x: f32, y: f32, a: f32) -> f32 {
    x * (1. - a) + y * a
}

pub fn mix_vec3(x: Vec3, y: Vec3, a: f32) -> Vec3 {
    x * (1. - a) + y * a
}

pub fn interpolation(d1: f32, d2: f32, k: f32) -> f32 {
    (0.5 + 0.5 * (d2 - d1) / k).clamp(0., 1.)
}

pub fn smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0., 1.);

    return mix(d2, d1, h) - k * h * (1. - h);
}

pub fn pow_vec3(v: Vec3, c: Vec3) -> Vec3 {
    vec3(v.x.powf(c.x), v.y.powf(c.y), v.z.powf(c.z))
}

pub fn tri_planar_mapping(p: Vec3, n: Vec3, blending: f32, scale: f32, tex: &Texture) -> Vec3 {
    let xy = p.xy() * scale;
    let xz = p.xz() * scale;
    let yz = p.yz() * scale;

    let x = tex.from_uv(yz.x, yz.y); 
    let y = tex.from_uv(xz.x, xz.y); 
    let z = tex.from_uv(xy.x, xy.y); 

    let bw = n.abs().powf(blending);
    let bw = bw / (bw.x + bw.y + bw.z);
    x * bw.x + y * bw.y + z * bw.z
}
