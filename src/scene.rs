use std::time::Instant;

use glam::{vec3, Vec3, Vec4};

use glam::vec4;
use rand::rngs::ThreadRng;
use sdl2::libc::EOPNOTSUPP;

use crate::objects::{Material, MaterialType, Object3D, Texture};
use crate::ray::{Ray, RayHit, EPSILON};

#[derive(Clone, Default)]
pub struct Light {
    pub direction: Vec3,
    pub power: f32,
}

#[derive(Clone)]
pub struct Scene {
    pub light: Light,
    pub ambient_color: Vec3,
    pub objects: Vec<Object3D>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub difuse: bool,
    pub max_ray_bounces: u8,
    pub max_frames_rendering: u32,
    pub shadow_casting: bool,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            light: Default::default(),
            ambient_color: Default::default(),
            objects: Default::default(),
            materials: Default::default(),
            textures: Default::default(),
            difuse: Default::default(),
            max_ray_bounces: Default::default(),
            max_frames_rendering: 1000,
            shadow_casting: false,
        }
    }
}

impl Scene {
    pub fn new(objects: Vec<Object3D>, materials: Vec<Material>) -> Scene {
        Scene {
            light: Light {
                direction: vec3(1., -1., -1.).normalize(),
                power: 1.,
            },
            ambient_color: vec3(0.0, 0.0, 0.0),
            objects,
            materials,
            textures: vec![],
            difuse: false,
            max_ray_bounces: 5,
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
        s.light = light;
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

    fn trace_ray(&self, ray: Ray, start_time: Instant) -> Option<RayHit> {
        if self.objects.is_empty() {
            return None;
        }

        let mut closest_t = f32::MAX;

        let mut closest_hit: Option<RayHit> = None;

        for obj in self.objects.iter() {
            if let k @ Some(t) = ray.hit(&obj, start_time) {
                if t.distance > 0. && t.distance < closest_t {
                    closest_hit = k;
                    closest_t = t.distance;
                }
            }
        }

        closest_hit
    }

    fn make_light(
        &self,
        ray: &Ray,
        hit: &RayHit,
        light_color: Vec3,
        albedo: Vec3,
        material: &Material,
    ) -> Vec3 {
        if !self.difuse {
            ray.phong(&hit, &self.light, albedo, material)
        } else {
            //println!("light {} emission {} albedo {}", light_color, material.emission_power, albedo);
            light_color + material.emission_power * albedo
        }
    }

    fn color(
        &self,
        ray: Ray,
        rnd: &mut ThreadRng,
        depth: u8,
        light_color: Vec3,
        contribution: Vec3,
        start_time: Instant,
    ) -> Vec3 {
        if depth >= self.max_ray_bounces {
            return light_color;
        }
        if let Some(hit) = self.trace_ray(ray, start_time) {
            let material = self.materials[hit.material_index];
            let mut albedo = material.albedo;

            match material.kind {
                MaterialType::Reflective { roughness } => {
                    if let Some(idx) = material.texture {
                        albedo = self.textures[idx].baricentric_pixel(hit.u, hit.v);
                    }

                    let p_light = self.make_light(&ray, &hit, light_color, albedo, &material);

                    let r = ray.reflection_ray(hit, roughness, rnd, self.difuse);

                    let mut col = self.color(
                        r,
                        rnd,
                        depth + 1,
                        p_light,
                        contribution * albedo,
                        start_time,
                    );
                    

                    if self.shadow_casting {
                        if let Some(obj) = self.trace_ray(
                            Ray {
                                origin: hit.point + EPSILON * hit.normal,
                                direction: -self.light.direction,
                            },
                            start_time,
                        ) {
                            // in the shadow
                            col *= 0.5;
                        }
                    }
                    col
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
                            start_time,
                        );
                    }

                    let reflection_ray = Ray {
                        origin: hit.point + EPSILON * hit.normal,
                        direction: ray.reflect(hit.normal),
                    };

                    let p_light = self.make_light(&ray, &hit, light_color, albedo, &material);
                    let reflection_color = self.color(
                        reflection_ray,
                        rnd,
                        depth + 1,
                        p_light,
                        contribution * albedo,
                        start_time,
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

    pub fn pixel(&self, ray: Ray, rnd: &mut ThreadRng, start_time: Instant) -> Vec4 {
        let mut light = Vec3::ZERO; // BLACK

        let contribution = Vec3::ONE;

        light = self.color(ray, rnd, 0, light, contribution, start_time);

        vec4(light.x, light.y, light.z, 1.)
    }
}
