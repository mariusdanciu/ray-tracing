extern crate sdl2;

use std::borrow::BorrowMut;
use std::sync::BarrierWaitResult;
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

use app::App;
use camera::{Camera, CameraEvent};
use fontdue_sdl2::fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use fontdue_sdl2::fontdue::Font;
use fontdue_sdl2::FontTexture;
use glam::{vec3, vec4, Vec3, Vec4};
use rand::seq::index::IndexVecIter;
use rand::{rngs::ThreadRng, Rng, RngCore};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::timer::Timer;
use sdl2::video::{Window, WindowContext};

mod app;
mod camera;

#[derive(Debug, Clone)]
pub struct Scene {
    rnd: ThreadRng,
    light_dir: Vec3,
    ambient_color: Vec3,
    spheres: Vec<Sphere>,
    materials: Vec<Material>,
    accumulated: Vec<Vec4>,
    frame_index: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct Material {
    albedo: Vec3,
    roughness: f32,
    metallic: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vec3::ZERO,
            roughness: 1.0,
            metallic: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    position: Vec3,
    radius: f32,
    material_index: usize,
}

impl Sphere {
    fn new(origin: Vec3, radius: f32, material_index: usize) -> Sphere {
        Sphere {
            position: origin,
            radius,
            material_index,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

#[derive(Debug, Copy, Clone)]
pub struct RayHit {
    object_index: usize,
    distance: f32,
    point: Vec3,
    normal: Vec3,
}

impl Scene {
    fn to_rgba(c: Vec4) -> (u8, u8, u8, u8) {
        (
            (c.x * 255.) as u8,
            (c.y * 255.) as u8,
            (c.z * 255.) as u8,
            (c.w + 255.) as u8,
        )
    }

    fn trace_ray(&mut self, ray: Ray) -> Option<RayHit> {
        // (bx^2 + by^2)t^2 + (2(axbx + ayby))t + (ax^2 + ay^2 - r^2) = 0
        // where
        // a = ray origin
        // b = ray direction
        // r = radius
        // t = hit distance

        if self.spheres.is_empty() {
            return None;
        }

        let mut closest_sphere = &self.spheres[0];
        let mut closest_t = f32::MIN;
        let mut closest_index: usize = usize::MAX;

        for (i, sphere) in self.spheres.iter().enumerate() {
            let origin = ray.origin - sphere.position;

            let a = ray.direction.dot(ray.direction);
            let b = 2. * origin.dot(ray.direction);
            let c = origin.dot(origin) - sphere.radius * sphere.radius;

            let disc = b * b - 4. * a * c;

            if disc < 0.0 {
                continue;
            }

            // closest to ray origin
            let t = (-b + disc.sqrt()) / (2.0 * a);
            //let t1 = (-b + disc.sqrt()) / (2.0 * a);

            if t < 0. && t > closest_t {
                closest_t = t;
                closest_sphere = sphere;
                closest_index = i;
            }
        }

        if closest_index == usize::MAX {
            return None;
        }

        let origin = ray.origin - closest_sphere.position; // translation
        let hit_point = origin + ray.direction * closest_t;

        let normal = hit_point.normalize();

        Some(RayHit {
            object_index: closest_index,
            distance: closest_t,
            point: hit_point + closest_sphere.position, // translation cancel
            normal,
        })
    }

    fn reflect(&self, incident: Vec3, normal: Vec3) -> Vec3 {
        incident - (2. * (incident.dot(normal))) * normal
    }

    fn pixel(&mut self, ray: Ray) -> Vec4 {
        let mut final_color = Vec3::new(0., 0., 0.);

        let mut factor = 1f32;
        let mut r = ray.clone();

        for i in 0..5 {
            if let Some(hit) = self.trace_ray(r) {
                let light = hit.normal.dot(-self.light_dir).max(0.0);

                let material = self.materials[self.spheres[hit.object_index].material_index];
                let color = material.albedo * light;

                final_color += color * factor;

                r.origin = hit.point + hit.normal * 0.0001;

                let mut rnd = self.rnd.clone();

                r.direction = self
                    .reflect(
                        r.direction,
                        hit.normal
                            + material.roughness
                                * vec3(
                                    rnd.gen_range(-0.5..0.5),
                                    rnd.gen_range(-0.5..0.5),
                                    rnd.gen_range(-0.5..0.5),
                                ),
                    )
                    .normalize();
            } else {
                final_color += self.ambient_color * factor;
                break;
            }
            factor *= 0.5;
        }

        vec4(final_color.x, final_color.y, final_color.z, 1.)
    }

    fn render(
        texture: &mut Texture,
        camera: &Camera,
        scene: &mut Scene,
        time_step: f32,
        updated: bool,
    ) -> Result<(), String> {
        let q = texture.query();
        let w = q.width as usize;
        let h = q.height as usize;

        if updated {
            scene.accumulated = vec![Vec4::ZERO; w * h];
            scene.frame_index = 1;
        }

        texture.set_blend_mode(sdl2::render::BlendMode::Blend);
        texture.set_alpha_mod(255);

        texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
            //let time = Instant::now();

            let mut i = 0;
            let mut pos = 0;

            for ray_dir in camera.ray_directions.iter() {
                //let t = Instant::now();
                let vcolor = scene.pixel(Ray {
                    origin: camera.position,
                    direction: *ray_dir,
                });

                scene.accumulated[pos] += vcolor;

                let mut accumulated = scene.accumulated[pos];
                accumulated /= scene.frame_index as f32;

                accumulated = accumulated.clamp(Vec4::ZERO, Vec4::ONE);
                
                let color = Scene::to_rgba(accumulated);
                buffer[i] = color.0;
                buffer[i + 1] = color.1;
                buffer[i + 2] = color.2;
                buffer[i + 3] = color.3;

                i += 4;
                pos += 1;
            }
        })?;

        scene.frame_index += 1;

        Ok(())
    }
}
pub fn main() -> Result<(), String> {
    let mut scene = Scene {
        frame_index: 1,
        rnd: rand::thread_rng(),
        light_dir: vec3(-1., -1., -1.).normalize(),
        ambient_color: vec3(0.6, 0.8, 1.0),
        accumulated: vec![],
        spheres: vec![
            Sphere::new(Vec3::new(0., 0., 0.), 0.5, 0),
            Sphere::new(Vec3::new(0., -100.5, 0.), 100., 1),
        ],
        materials: vec![
            Material {
                albedo: Vec3::new(0., 0.5, 0.7),
                roughness: 1.,
                metallic: 0.0,
            },
            Material {
                albedo: Vec3::new(0.8, 0.8, 0.8),
                roughness: 1.,
                metallic: 0.0,
            },
        ],
    };

    App::run(&mut scene, Scene::render)
}
