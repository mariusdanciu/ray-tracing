use glam::{vec3, Vec3, Vec4};

use glam::vec4;
use rand::rngs::ThreadRng;

use crate::objects::{Material, MaterialType, Object3D, Texture};
use crate::ray::{Ray, RayHit};

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

    fn make_light(&self, albedo: Vec3, emission_power: f32, light: Vec3, light_angle: f32) -> Vec3 {
        if !self.difuse {
            albedo * light_angle
        } else {
            light + albedo * emission_power
        }
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
            let light_angle = hit.normal.dot(-self.light.direction).max(0.0) * self.light.power;

            match material.kind {
                MaterialType::Reflective { roughness } => {
                    if let Some(idx) = material.texture {
                        albedo = self.textures[idx].baricentric_pixel(hit.u, hit.v);
                    }
                    let light =
                        self.make_light(albedo, material.emission_power, light, light_angle);

                    let r = ray.reflection_ray(hit, roughness, rnd);
                    self.color(r, rnd, depth + 1, light, contribution * albedo)
                }
                MaterialType::Refractive {
                    transparency,
                    refraction_index,
                } => {
                    let light =
                        self.make_light(albedo, material.emission_power, light, light_angle);
                    let mut refraction_color = Vec3::ZERO;
                    let kr = material.fresnel(ray.direction, hit.normal, refraction_index) as f32;

                    if kr < 1.0 {
                        if let Some(refraction_ray) = ray.refraction_ray(hit, refraction_index) {
                            refraction_color = self.color(
                                refraction_ray,
                                rnd,
                                depth + 1,
                                light,
                                contribution * albedo,
                            );
                        }
                    }

                    let outside = ray.direction.dot(hit.normal) < 0.;
                    let bias = 0.0001 * hit.normal;
                    let orig: Vec3 = if outside {
                        hit.point + bias
                    } else {
                        hit.point - bias
                    };

                    let reflection_ray = Ray {
                        origin: orig,
                        direction: ray.reflect(hit.normal).normalize(),
                    };

                    let _reflection_ray = ray.reflection_ray(hit, 0., rnd);

                    let reflection_color =
                        self.color(reflection_ray, rnd, depth + 1, light, contribution * albedo);

                    let color =
                        reflection_color * kr + refraction_color * (1.0 - kr) * transparency;
                    color * albedo
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
