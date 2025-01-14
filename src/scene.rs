use glam::{vec3, Vec3, Vec4};

use glam::vec4;
use rand::rngs::ThreadRng;

use crate::objects::{Material, MaterialType, Object3D, Texture};
use crate::ray::{Ray, RayHit};

#[derive(Clone)]
pub struct Scene {
    pub light_dir: Vec3,
    pub ambient_color: Vec3,
    pub objects: Vec<Object3D>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub difuse: bool,
    pub max_ray_bounces: u8,
    pub max_frames_rendering: u32,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            light_dir: Default::default(),
            ambient_color: Default::default(),
            objects: Default::default(),
            materials: Default::default(),
            textures: Default::default(),
            difuse: Default::default(),
            max_ray_bounces: Default::default(),
            max_frames_rendering: 1000,
        }
    }
}

impl Scene {
    pub fn new(objects: Vec<Object3D>, materials: Vec<Material>) -> Scene {
        Scene {
            light_dir: vec3(-1., -1., -1.).normalize(),
            ambient_color: vec3(0.1, 0.1, 0.1),
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

    fn trace_ray(&self, ray: Ray) -> Option<RayHit> {
        if self.objects.is_empty() {
            return None;
        }

        let mut closest_t = f32::MIN;

        let mut closest_hit: Option<RayHit> = None;

        for obj in self.objects.iter() {
            if let k @ Some(t) = ray.hit(&obj) {
                if t.distance < 0. && t.distance > closest_t {
                    closest_hit = k;
                    closest_t = t.distance;
                }
            }
        }

        closest_hit
    }

    fn color(
        &self,
        ray: Ray,
        rnd: &mut ThreadRng,
        depth: u8,
        light: Vec3,
        contribution: Vec3,
    ) -> Vec3 {
        if depth >= self.max_ray_bounces {
            return light;
        }
        if let Some(hit) = self.trace_ray(ray) {
            let material = self.materials[hit.material_index];
            let mut albedo = material.albedo;
            let light_angle = hit.normal.dot(-self.light_dir).max(0.0);

            let light_multiplier: f32;
            if !self.difuse {
                light_multiplier = light_angle;
            } else {
                light_multiplier = material.emission_power;
            }

            match material.kind {
                MaterialType::Reflective { roughness } => {
                    let mut ll = light;
                    if let Some(idx) = material.texture {
                        albedo = self.textures[idx].baricentric_pixel(hit.u, hit.v);
                    }

                    ll += albedo * light_multiplier;

                    let r = ray.reflection_ray(hit, roughness, rnd);
                    self.color(r, rnd, depth + 1, ll, contribution * albedo)
                }
                MaterialType::Refractive {
                    transparency,
                    refraction_index,
                } => {
                    let mut refraction_color = Vec3::ZERO;
                    let kr = material.fresnel(ray.direction, hit.normal, refraction_index) as f32;

                    if kr < 1.0 {
                        if let Some(refraction_ray) = ray.refraction_ray(hit, refraction_index) {
                            refraction_color = self.color(
                                refraction_ray,
                                rnd,
                                depth + 1,
                                light + albedo * light_multiplier,
                                contribution * albedo,
                            );
                        }
                    }

                    let outside = ray.direction.dot(hit.normal) < 0.;
                    let bias = 0.0001 * hit.normal;
                    let orig: Vec3;
                    if outside {
                        orig = hit.point + bias;
                    } else {
                        orig = hit.point - bias;
                    }
                    let reflection_ray = Ray {
                        origin: orig,
                        direction: ray.reflect(hit.normal).normalize(),
                    };

                    let reflection_color = self.color(
                        reflection_ray,
                        rnd,
                        depth + 1,
                        light + albedo * light_multiplier,
                        contribution * albedo,
                    );

                    let color = reflection_color * kr + refraction_color * (1.0 - kr);
                    light + color * transparency * albedo * light_multiplier
                }
            }
        } else {
            light + self.ambient_color * contribution
        }
    }

    pub fn pixel(&self, ray: Ray, rnd: &mut ThreadRng) -> Vec4 {
        let mut light = Vec3::ZERO; // BLACK

        let contribution = Vec3::ONE;

        light = self.color(ray, rnd, 0, light, contribution);

        vec4(light.x, light.y, light.z, 1.)
    }
}
