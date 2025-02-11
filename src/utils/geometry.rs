use glam::{vec3, Mat4, Vec3};

pub static DEGREES: f32 = std::f32::consts::PI / 180.;

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