use glam::{vec3, Vec3, Vec4};

use glam::vec4;
use rand::rngs::ThreadRng;

use crate::light::Light;
use crate::objects::{Material, Object3D, Texture};
use crate::ray::Ray;
use crate::ray_marching::RayMarching;
use crate::ray_tracing::RayTracing;

#[derive(Debug, Clone)]
pub struct Scene {
    pub lights: Vec<Light>,
    pub ambient_color: Vec3,
    pub objects: Vec<Object3D>,
    pub sdfs: Vec<usize>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub max_ray_bounces: u8,

    pub shadow_casting: bool,
    pub ray_marching: bool,
    pub diffuse: bool,
    pub enable_accumulation: bool,

    pub update_func: Option<fn(&mut Scene, f32) -> bool>,

}

impl Default for Scene {
    fn default() -> Self {
        Self {
            lights: vec![],
            ambient_color: Default::default(),
            objects: Default::default(),
            sdfs: Default::default(),
            materials: Default::default(),
            textures: Default::default(),
            max_ray_bounces: Default::default(),
            shadow_casting: false,
            ray_marching: false,
            diffuse: false,
            enable_accumulation: false,
            update_func: None,
        }
    }
}

impl Scene {
    pub fn new(objects: Vec<Object3D>, materials: Vec<Material>) -> Scene {
        Scene {
            ambient_color: vec3(0.0, 0.0, 0.0),
            objects,
            materials,
            textures: vec![],
            max_ray_bounces: 4,
            ..Default::default()
        }
    }

    pub fn with_light(&self, light: Light) -> Scene {
        let mut s = self.clone();
        s.lights.push(light);
        s
    }

    pub fn with_texture(&self, texture: Texture) -> Scene {
        let mut s = self.clone();
        s.textures.push(texture);
        s
    }

    pub fn with_textures(&self, mut textures: Vec<Texture>) -> Scene {
        let mut s = self.clone();
        s.textures.append(&mut textures);
        s
    }

    pub fn pixel(&self, ray: Ray, rnd: &mut ThreadRng) -> Vec4 {
        let light = if !self.ray_marching {
            let tracer = RayTracing { scene: self };
            tracer.albedo(ray, rnd)
        } else {
            let tracer = RayMarching { scene: self };
            tracer.albedo(ray, rnd)
        };

        vec4(light.x, light.y, light.z, 1.)
    }
}
