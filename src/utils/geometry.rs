use glam::Mat4;


pub fn rotate_y_mat(o: f32) -> Mat4 {
    let (s, c) = f32::sin_cos(o);
    Mat4::from_cols_array(&[c, 0., s, 0., 0., 1., 0., 0., -s, 0., c, 0., 0., 0., 0., 1.])
}

pub fn rotate_x_mat(o: f32) -> Mat4 {
    let (s, c) = f32::sin_cos(o);
    Mat4::from_cols_array(&[1., 0., 0., 0., 0., c, -s, 0., 0., s, c, 0., 0., 0., 0., 1.])
}