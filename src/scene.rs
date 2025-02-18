use glam::{vec3, Vec3, Vec4};

use glam::vec4;
use rand::rngs::ThreadRng;

use crate::objects::{Material, MaterialType, Object3D, Texture};
use crate::ray::{Ray, RayHit, EPSILON};

#[derive(Debug, Clone)]
pub enum Light {
    Directional { direction: Vec3, intensity: f32 },
    Positional { position: Vec3, intensity: f32 },
    Spot { position: Vec3, intensity: f32 },
}

impl Light {
    pub fn direction(&self, point: Vec3) -> Vec3 {
        match *self {
            Light::Directional { direction, .. } => direction,
            Light::Positional { position, .. } => (point - position).normalize(),
            Light::Spot { position, .. } => (point - position).normalize(),
        }
    }

    pub fn distance(&self, point: Vec3) -> f32 {
        match *self {
            Light::Directional { direction, .. } => 1.,
            Light::Positional { position, .. } => (point - position).length(),
            Light::Spot { position, .. } => (point - position).length(),
        }
    }
    pub fn intensity(&self) -> f32 {
        match *self {
            Light::Directional {
                direction,
                intensity,
            } => intensity,
            Light::Positional {
                position,
                intensity,
            } => intensity,
            Light::Spot {
                position,
                intensity,
            } => intensity,
        }
    }
}

#[derive(Clone)]
pub struct Scene {
    pub lights: Vec<Light>,
    pub ambient_color: Vec3,
    pub objects: Vec<Object3D>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub diffuse: bool,
    pub enable_accumulation: bool,
    pub max_ray_bounces: u8,
    pub max_frames_rendering: u32,
    pub shadow_casting: bool,
    pub update_func: Option<fn(&mut Scene, f32) -> bool>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            lights: vec![],
            ambient_color: Default::default(),
            objects: Default::default(),
            materials: Default::default(),
            textures: Default::default(),
            diffuse: Default::default(),
            max_ray_bounces: Default::default(),
            max_frames_rendering: 1000,
            shadow_casting: false,
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
            diffuse: false,
            max_ray_bounces: 4,
            ..Default::default()
        }
    }

    pub fn to_rgba(c: Vec4) -> (u8, u8, u8, u8) {
        (
            (c.x * 255.) as u8,
            (c.y * 255.) as u8,
            (c.z * 255.) as u8,
            (c.w + 255.) as u8,
        )
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

    fn trace_ray(&self, ray: Ray) -> Option<(RayHit, usize)> {
        if self.objects.is_empty() {
            return None;
        }

        let mut closest_t = f32::MAX;

        let mut closest_hit: Option<(RayHit, usize)> = None;

        for (idx, obj) in self.objects.iter().enumerate() {
            if let Some(t) = ray.hit(&obj) {
                if t.distance > 0. && t.distance < closest_t {
                    closest_hit = Some((t, idx));
                    closest_t = t.distance;
                }
            }
        }

        closest_hit
    }

    fn color_diffuse(
        &self,
        ray: Ray,
        rnd: &mut ThreadRng,
        depth: u8,
        light_color: Vec3,
        contribution: Vec3,
        time: f32,
    ) -> Vec3 {
        if depth >= self.max_ray_bounces {
            return light_color;
        }
        if let Some((hit, obj_index)) = self.trace_ray(ray) {
            let material = self.materials[hit.material_index];
            let mut albedo = material.albedo;

            match material.kind {
                MaterialType::Reflective { roughness } => {
                    if let Some(idx) = material.texture {
                        albedo = self.textures[idx].from_uv(hit.u, hit.v);
                    }

                    let p_light = light_color + material.emission_power * albedo;

                    let r = ray.reflection_ray(
                        hit,
                        roughness,
                        rnd,
                        self.diffuse,
                        self.enable_accumulation,
                    );

                    let reflected_col =
                        self.color_diffuse(r, rnd, depth + 1, p_light, contribution * albedo, time);

                    reflected_col
                }
                MaterialType::Refractive {
                    transparency,
                    refraction_index,
                    reflectivity,
                } => {
                    let mut refraction_color = Vec3::ZERO;
                    let kr =
                        material.fresnel(ray.direction, hit.normal, refraction_index, reflectivity)
                            as f32;

                    if let Some(refraction_ray) = ray.refraction_ray(hit, refraction_index) {
                        refraction_color = self.color(
                            refraction_ray,
                            rnd,
                            depth + 1,
                            light_color,
                            contribution * albedo,
                            time,
                        );
                    }

                    let reflection_ray = Ray {
                        origin: hit.point + EPSILON * hit.normal,
                        direction: ray.reflect(hit.normal),
                    };

                    let p_light = light_color + material.emission_power * albedo;
                    let reflection_color = self.color_diffuse(
                        reflection_ray,
                        rnd,
                        depth + 1,
                        p_light,
                        contribution * albedo,
                        time,
                    );

                    let color =
                        reflection_color * kr + refraction_color * (1.0 - kr) * transparency;
                    color
                }
            }
        } else {
            light_color + self.ambient_color * contribution
        }
    }

    fn light(
        &self,
        ray: &Ray,
        hit: &RayHit,
        albedo: Vec3,
        material: &Material,
        obj_index: usize,
    ) -> Vec3 {
        let mut l_acc = Vec3::ZERO;
        for l in &self.lights {
            let k = ray.blinn_phong(&hit, l, albedo, material);
            let light_dis = l.distance(hit.point);
            l_acc += (k / (light_dis * light_dis)) * l.intensity();

            if self.shadow_casting {
                if let Some((hit, idx)) = self.trace_ray(Ray {
                    origin: hit.point + EPSILON * hit.normal,
                    direction: -l.direction(hit.point),
                }) {
                    if idx != obj_index {
                        // in the shadow
                        l_acc *= 0.5;
                    }
                }
            }
        }
        l_acc
    }

    fn color(
        &self,
        ray: Ray,
        rnd: &mut ThreadRng,
        depth: u8,
        light_color: Vec3,
        contribution: Vec3,
        time: f32,
    ) -> Vec3 {
        if depth >= self.max_ray_bounces {
            return light_color;
        }
        if let Some((hit, obj_index)) = self.trace_ray(ray) {
            let material = self.materials[hit.material_index];
            let mut albedo = material.albedo;

            match material.kind {
                MaterialType::Reflective { roughness } => {
                    if let Some(idx) = material.texture {
                        albedo = self.textures[idx].from_uv(hit.u, hit.v);
                    }

                    let p_light = self.light(&ray, &hit, albedo, &material, obj_index);

                    let r = ray.reflection_ray(
                        hit,
                        roughness,
                        rnd,
                        self.diffuse,
                        self.enable_accumulation,
                    );

                    let reflected_col =
                        self.color(r, rnd, depth + 1, p_light, contribution * albedo, time);

                    p_light * (roughness) + p_light * reflected_col * (1. - roughness)
                }

                MaterialType::Refractive {
                    transparency,
                    refraction_index,
                    reflectivity,
                } => {
                    let mut refraction_color = Vec3::ZERO;
                    let kr =
                        material.fresnel(ray.direction, hit.normal, refraction_index, reflectivity)
                            as f32;

                    if let Some(refraction_ray) = ray.refraction_ray(hit, refraction_index) {
                        refraction_color = self.color(
                            refraction_ray,
                            rnd,
                            depth + 1,
                            light_color,
                            contribution * albedo,
                            time,
                        );
                    }

                    let reflection_ray = Ray {
                        origin: hit.point + EPSILON * hit.normal,
                        direction: ray.reflect(hit.normal),
                    };

                    let p_light = self.light(&ray, &hit, albedo, &material, obj_index);

                    let reflection_color = self.color(
                        reflection_ray,
                        rnd,
                        depth + 1,
                        p_light,
                        contribution * albedo,
                        time,
                    );

                    let color =
                        reflection_color * kr + refraction_color * (1.0 - kr) * transparency;

                    color * albedo
                }
            }
        } else {
            light_color + self.ambient_color * contribution
        }
    }

    pub fn pixel(&self, ray: Ray, rnd: &mut ThreadRng, time: f32) -> Vec4 {
        let mut light = Vec3::ZERO; // BLACK

        let contribution = Vec3::ONE;

        light = if self.diffuse {
            self.color_diffuse(ray, rnd, 0, light, contribution, time)
        } else {
            self.color(ray, rnd, 0, light, contribution, time)
        };
        vec4(light.x, light.y, light.z, 1.)
    }
}
