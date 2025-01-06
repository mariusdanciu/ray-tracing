use glam::{Mat4, Vec2, Vec3, Vec4};

#[derive(Debug, Clone)]
pub struct Camera {
    pub width: usize,
    pub height: usize,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub position: Vec3,
    pub forward_direction: Vec3,
    pub up: Vec3,
    pub view: Mat4,
    pub inverse_view: Mat4,
    pub perspective: Mat4,
    pub inverse_perspective: Mat4,
    pub ray_directions: Vec<Vec3>,
}

impl Default for Camera {
    fn default() -> Self {
        let pos = Vec3::new(0.0, 0.0, 3.);
        let dir = Vec3::new(0.0, 0.0, -1.);

        let up = Vec3::new(0., 1., 0.);
        let fov: f32 = 45.0;
        let near: f32 = -1.;
        let far: f32 = -100.;
        let view = Mat4::IDENTITY;
        let inverse_view = Mat4::IDENTITY;
        let perspective = Mat4::IDENTITY;
        let inverse_perspective = Mat4::IDENTITY;

        Self {
            width: 800,
            height: 600,
            fov,
            near,
            far,
            position: pos,
            forward_direction: dir,
            up,
            view,
            inverse_view,
            perspective,
            inverse_perspective,
            ray_directions: Vec::new(),
        }
    }
}
pub enum CameraEvent {
    Resize { w: usize, h: usize },
    RotateXY { delta: Vec2 },
    Up,
    Down,
    Left,
    Right,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            ..Default::default()
        }
    }

    pub fn new_with_pos(position: Vec3, look_at: Vec3) -> Camera {
        Camera {
            position,
            forward_direction: look_at,
            ..Default::default()
        }
    }

    pub fn rotate_y_mat(o: f32) -> Mat4 {
        let (s, c) = f32::sin_cos(o);
        Mat4::from_cols_array(&[c, 0., s, 0., 0., 1., 0., 0., -s, 0., c, 0., 0., 0., 0., 1.])
    }

    pub fn rotate_x_mat(o: f32) -> Mat4 {
        let (s, c) = f32::sin_cos(o);
        Mat4::from_cols_array(&[1., 0., 0., 0., 0., c, -s, 0., 0., s, c, 0., 0., 0., 0., 1.])
    }

    pub fn update(&mut self, event: CameraEvent) {
        let right_direction = self.forward_direction.cross(self.up);
        let speed = 0.2;
        let rotation_speed = 4.;
        match event {
            CameraEvent::Up => self.position += self.forward_direction * speed,
            CameraEvent::Down => self.position -= self.forward_direction * speed,
            CameraEvent::Left => self.position -= right_direction * speed,
            CameraEvent::Right => self.position += right_direction * speed,
            CameraEvent::Resize { w, h } => {
                self.width = w;
                self.height = h;
                self.perspective =
                    Mat4::perspective_rh(self.fov, w as f32 / h as f32, self.near, self.far);
                self.inverse_perspective = self.perspective.inverse();
            }

            CameraEvent::RotateXY { delta } => {
                let pitch_delta = delta.y * rotation_speed;
                let yaw_delta = delta.x * rotation_speed;

                let rotation =
                    Camera::rotate_x_mat(pitch_delta as f32 * std::f32::consts::PI / 180.)
                        * Camera::rotate_y_mat(yaw_delta as f32 * std::f32::consts::PI / 180.);

                let fd = rotation
                    * Vec4::new(
                        self.forward_direction.x,
                        self.forward_direction.y,
                        self.forward_direction.z,
                        1.,
                    );

                self.forward_direction = Vec3::new(fd.x, fd.y, fd.z);
            }
        }

        self.view = Mat4::look_at_rh(
            self.position,
            self.position + self.forward_direction,
            self.up,
        );

        self.inverse_view = self.view.inverse();

        self.calculate_ray_directions();
    }

    fn calculate_ray_directions(&mut self) {
        self.ray_directions = vec![Vec3::ZERO; (self.width * self.height) as usize];
        let mut y = 0;
        let mut x = 0;

        while y < self.height {
            while x < self.width {
                let p_ndc_x = (x as f32) / self.width as f32;
                let p_ndc_y = (y as f32) / self.height as f32;

                let p_screen_x = 2.0 * p_ndc_x - 1.;
                let p_screen_y = 1. - 2.0 * p_ndc_y;

                let target = self.inverse_perspective * Vec4::new(p_screen_x, p_screen_y, 1., 1.);
                let v3 = Vec3::new(target.x, target.y, target.z) / target.w;
                let ray_direction = self.inverse_view * Vec4::new(v3.x, v3.y, v3.z, 0.0);

                self.ray_directions[x + y * self.width as usize] =
                    Vec3::new(ray_direction.x, ray_direction.y, ray_direction.z).normalize();

                x += 1;
            }
            y += 1;
            x = 0;
        }
    }
}
