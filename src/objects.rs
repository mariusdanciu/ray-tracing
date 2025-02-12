use std::time::Instant;

use glam::{vec3, Mat4, Vec2, Vec3, Vec4};

use crate::utils::geometry;

#[derive(Debug, Copy, Clone)]
pub enum Object3D {
    Sphere {
        position: Vec3,
        rotation_axis: Vec3,
        radius: f32,
        material_index: usize,
        transform: Mat4,
        inv_transform: Mat4,
    },

    Triangle {
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        material_index: usize,
    },

    Box {
        position: Vec3,
        rotation_axis: Vec3,
        transform: Mat4,
        inv_transform: Mat4,
        dimension: Vec3,
        material_index: usize,
    },

    Plane {
        normal: Vec3,
        point: Vec3,
        max_dist: Option<Vec2>,
        material_index: usize,
    },

    Cylinder {
        radius: f32,
        height: f32,
        position: Vec3,
        rotation_axis: Vec3,
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

    fn textel(&self, p: f32) -> f32 {
        if p < 0. {
            return 1. - (p.ceil() - p).abs();
        } else if p > 1. {
            return (p - p.floor());
        }
        p
    }

    pub fn from_uv(&self, u: f32, v: f32) -> Vec3 {
        let uu = self.textel(u);
        let vv = self.textel(v);

        if uu > 1. || vv > 1. {
            println!("UV {} {} ", uu, vv);
        }
        let x = ((self.width - 1) as f32 * uu) as u32;
        let y = ((self.height - 1) as f32 * vv) as u32;
        self.pixel(x, y)
    }

    pub fn pixel(&self, x: u32, y: u32) -> Vec3 {
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
        let mut cos_x = normal.dot(-incident);
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
}

impl Object3D {
    pub fn new_plane(
        normal: Vec3,
        point: Vec3,
        material_index: usize,
        max_dist: Option<Vec2>,
    ) -> Object3D {
        Object3D::Plane {
            normal,
            point,
            max_dist,
            material_index,
        }
    }
    pub fn new_sphere(origin: Vec3, radius: f32, material_index: usize) -> Object3D {
        let t = Mat4::from_translation(origin);
        Object3D::Sphere {
            position: origin,
            rotation_axis: Vec3::ZERO,
            radius,
            material_index,
            transform: t,
            inv_transform: t.inverse(),
        }
    }

    pub fn new_sphere_with_rotation(
        origin: Vec3,
        rotation_axis: Vec3,
        radius: f32,
        material_index: usize,
    ) -> Object3D {
        let t = Mat4::from_translation(origin)
            * Mat4::from_rotation_x(rotation_axis.x * geometry::DEGREES)
            * Mat4::from_rotation_y(rotation_axis.y * geometry::DEGREES)
            * Mat4::from_rotation_z(rotation_axis.z * geometry::DEGREES);
        Object3D::Sphere {
            position: origin,
            rotation_axis,
            radius,
            material_index,
            transform: t,
            inv_transform: t.inverse(),
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

    pub fn new_box(
        position: Vec3,
        rotation_axis: Vec3,
        dimension: Vec3,
        material_index: usize,
    ) -> Object3D {
        let t = Mat4::from_translation(position)
            * Mat4::from_rotation_x(rotation_axis.x * geometry::DEGREES)
            * Mat4::from_rotation_y(rotation_axis.y * geometry::DEGREES)
            * Mat4::from_rotation_z(rotation_axis.z * geometry::DEGREES);
        Object3D::Box {
            position,
            rotation_axis,
            transform: t,
            inv_transform: t.inverse(),
            dimension,
            material_index,
        }
    }
}
