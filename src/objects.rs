use glam::{vec3, Vec3, Vec4};

#[derive(Debug, Copy, Clone)]
pub enum Object3D {
    Sphere {
        position: Vec3,
        radius: f32,
        material_index: usize,
    },

    Triangle {
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        material_index: usize,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum MaterialType {
    Reflective {
        roughness: f32,
    },
    Refractive {
        transparency: f32,
        refraction_index: f32,
        reflectivity: f32,
    },
}

#[derive(Debug, Copy, Clone)]
pub struct Cuboid {
    pub center: Vec3,
    pub length: f32,
    pub width: f32,
    pub depth: f32,
}

impl Cuboid {
    pub fn triangles(&self, material_index: usize) -> Vec<Object3D> {
        // Face 1
        let v1_1 = vec3(
            self.center.x - self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        let v1_2 = vec3(
            self.center.x - self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        let v1_3 = vec3(
            self.center.x + self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        let v1_4 = vec3(
            self.center.x + self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        // Face 2

        let v2_1 = vec3(
            self.center.x - self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        let v2_2 = vec3(
            self.center.x - self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        let v2_3 = vec3(
            self.center.x - self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        let v2_4 = vec3(
            self.center.x - self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        // Face 3

        let v3_1 = vec3(
            self.center.x - self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        let v3_2 = vec3(
            self.center.x - self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        let v3_3 = vec3(
            self.center.x + self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        let v3_4 = vec3(
            self.center.x + self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        // Face 4

        let v4_1 = vec3(
            self.center.x + self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        let v4_2 = vec3(
            self.center.x + self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        let v4_3 = vec3(
            self.center.x + self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        let v4_4 = vec3(
            self.center.x + self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        // Face 5

        let v5_1 = vec3(
            self.center.x - self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        let v5_2 = vec3(
            self.center.x + self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        let v5_3 = vec3(
            self.center.x + self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        let v5_4 = vec3(
            self.center.x - self.length / 2.,
            self.center.y + self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        // Face 6

        let v6_1 = vec3(
            self.center.x - self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        let v6_2 = vec3(
            self.center.x + self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z + self.depth / 2.,
        );

        let v6_3 = vec3(
            self.center.x + self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        let v6_4 = vec3(
            self.center.x - self.length / 2.,
            self.center.y - self.width / 2.,
            self.center.z - self.depth / 2.,
        );

        vec![
            Object3D::new_triangle(v1_1, v1_2, v1_3, material_index),
            Object3D::new_triangle(v1_1, v1_3, v1_4, material_index),
            Object3D::new_triangle(v2_1, v2_2, v2_3, material_index),
            Object3D::new_triangle(v2_1, v2_3, v2_4, material_index),
            Object3D::new_triangle(v3_1, v3_2, v3_3, material_index),
            Object3D::new_triangle(v3_1, v3_3, v3_4, material_index),
            Object3D::new_triangle(v4_1, v4_2, v4_3, material_index),
            Object3D::new_triangle(v4_1, v4_3, v4_4, material_index),
            Object3D::new_triangle(v5_1, v5_2, v5_3, material_index),
            Object3D::new_triangle(v5_1, v5_3, v5_4, material_index),
            Object3D::new_triangle(v6_1, v6_2, v6_3, material_index),
            Object3D::new_triangle(v6_1, v6_3, v6_4, material_index),
        ]
    }
}

#[derive(Default, Debug, Clone)]
pub struct Texture {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

impl Texture {
    pub fn new(path: impl Into<String>) -> Texture {
        Texture {
            path: path.into(),
            ..Default::default()
        }
    }

    pub fn baricentric_pixel(&self, u: f32, v: f32) -> Vec3 {
        let x = (self.width as f32 * u) as u32;
        let y = (self.height as f32 * v) as u32;
        self.pixel(x, y)
    }

    pub fn pixel(&self, x: u32, y: u32) -> Vec3 {
        //println!("x {} y {} w {}, h {}", x, y, self.width, self.height);
        let pos = (y * 3 * self.width + x * 3) as usize;

        Vec3::new(
            (self.bytes[pos] as f32) / 255.,
            (self.bytes[pos + 1] as f32) / 255.,
            (self.bytes[pos + 2] as f32) / 255.,
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub ambience: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub shininess: f32,
    pub albedo: Vec3,
    pub texture: Option<usize>,
    pub kind: MaterialType,
    pub emission_power: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            ambience: 0.2,
            diffuse: 0.7,
            specular: 0.5,
            shininess: 5.,
            albedo: Vec3::ZERO,
            texture: None,
            kind: MaterialType::Reflective { roughness: 1.0 },
            emission_power: 0.0,
        }
    }
}

impl Material {
    pub fn fresnel(
        &self,
        incident: Vec3,
        normal: Vec3,
        refraction_index: f32,
        reflectivity: f32,
    ) -> f32 {
        let n2 = refraction_index;
        let n1 = 1.0;

        let mut r0 = (n1 - n2) / (n1 + n2);
        r0 *= r0;
        let mut cos_x = -normal.dot(incident);
        if n1 > n2 {
            let n = n1 / n2;
            let sin_t2 = n * n * (1.0 - cos_x * cos_x);
            // Total internal reflection
            if sin_t2 > 1.0 {
                return 1.0;
            }
            cos_x = (1.0 - sin_t2).sqrt();
        }
        let x = 1.0 - cos_x;
        let ret = r0 + (1.0 - r0) * x * x * x * x * x;

        // adjust reflect multiplier for object reflectivity
        reflectivity + (1.0 - reflectivity) * ret
    }

    pub fn _fresnel(&self, incident: Vec3, normal: Vec3, index: f32) -> f32 {
        let mut i_dot_n = incident.dot(normal).clamp(-1., 1.);
        let mut eta_i = 1.0;
        let mut eta_t = index;
        if i_dot_n < 0.0 {
            i_dot_n = -i_dot_n;
        } else {
            eta_i = eta_t;
            eta_t = 1.0;
        }
        let eta = eta_i / eta_t;

        let sin_t = eta * (1. - i_dot_n * i_dot_n).max(0.0).sqrt();
        if sin_t > 1.0 {
            //Total internal reflection
            return 1.0;
        } else {
            let cos_t = (1.0 - sin_t * sin_t).max(0.0).sqrt();
            let cos_i = cos_t.abs();

            let et_ci = eta_t * cos_i;
            let ei_ct = eta_i * cos_t;
            let ei_ci = eta_i * cos_i;
            let et_ct = eta_t * cos_t;

            let r_s = (et_ci - ei_ct) / (et_ci + ei_ct);
            let r_p = (ei_ci - et_ct) / (ei_ci + et_ct);
            return (r_s * r_s + r_p * r_p) / 2.0;
        }
    }
}

impl Object3D {
    pub fn new_sphere(origin: Vec3, radius: f32, material_index: usize) -> Object3D {
        Object3D::Sphere {
            position: origin,
            radius,
            material_index,
        }
    }
    pub fn new_triangle(v1: Vec3, v2: Vec3, v3: Vec3, material_index: usize) -> Object3D {
        Object3D::Triangle {
            v1,
            v2,
            v3,
            material_index,
        }
    }
}
