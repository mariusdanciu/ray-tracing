extern crate sdl2;

use std::borrow::BorrowMut;
use std::sync::BarrierWaitResult;
use std::thread::Thread;
use std::time::{Duration, Instant};

use app::App;
use camera::{Camera, CameraEvent};
use fontdue_sdl2::fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use fontdue_sdl2::fontdue::Font;
use fontdue_sdl2::FontTexture;
use glam::{vec3, Vec3};
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
pub struct State {
    light_dir: Vec3,
    spheres: Vec<Sphere>,
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    position: Vec3,
    radius: f32,
    color: Vec3,
}

impl Sphere {
    fn new(origin: Vec3, radius: f32, color: Vec3) -> Sphere {
        Sphere {
            position: origin,
            radius,
            color,
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

fn to_rgba(c: Vec3) -> (u8, u8, u8, u8) {
    (
        (c.x * 255.) as u8,
        (c.y * 255.) as u8,
        (c.z * 255.) as u8,
        (255.) as u8,
    )
}

fn trace_ray(ray: Ray, state: &mut State) -> Option<RayHit> {
    // (bx^2 + by^2)t^2 + (2(axbx + ayby))t + (ax^2 + ay^2 - r^2) = 0
    // where
    // a = ray origin
    // b = ray direction
    // r = radius
    // t = hit distance

    if state.spheres.is_empty() {
        return None;
    }

    let mut closest_sphere = &state.spheres[0];
    let mut closest_t = f32::MIN;
    let mut closest_index: usize = usize::MAX;

    for (i, sphere) in state.spheres.iter().enumerate() {
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

    let origin = ray.origin - closest_sphere.position;
    let hit_point = origin + ray.direction * closest_t;

    let normal = hit_point.normalize();

    Some(RayHit {
        object_index: closest_index,
        distance: closest_t,
        point: hit_point + closest_sphere.position,
        normal,
    })
}

fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - (2. * (incident.dot(normal))) * normal
}

fn pixel(ray: Ray, state: &mut State) -> (u8, u8, u8, u8) {
    let mut final_color = Vec3::new(0., 0., 0.);

    let bk = Vec3::ZERO;
    let mut factor = 1f32;
    let mut r = ray.clone();

    for i in 0..2 {
        if let Some(hit) = trace_ray(r, state) {
            let light = hit.normal.dot(-state.light_dir).max(0.0);

            let color = state.spheres[hit.object_index].color * light;

            final_color += color * factor;

            r.origin = hit.point + hit.normal * 0.0001;
            r.direction = reflect(r.direction, hit.normal).normalize();
        } else {
            //final_color += bk * factor;
            break;
        }
        factor *= 0.5;
    }

    to_rgba(final_color)
}

fn render(
    texture: &mut Texture,
    camera: &Camera,
    state: &mut State,
    time_step: f32,
) -> Result<(), String> {
    let q = texture.query();
    let w = q.width as usize;
    let h = q.height as usize;

    texture.set_blend_mode(sdl2::render::BlendMode::Blend);
    texture.set_alpha_mod(255);

    texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
        //let time = Instant::now();

        let mut i = 0;

        for ray_dir in camera.ray_directions.iter() {
            let color = pixel(
                Ray {
                    origin: camera.position,
                    direction: *ray_dir,
                },
                state,
            );

            buffer[i] = color.0;
            buffer[i + 1] = color.1;
            buffer[i + 2] = color.2;
            buffer[i + 3] = color.3;

            i += 4;
        }
        //println!("update texture {:?}", time.elapsed());
    })?;

    Ok(())
}

pub fn main() -> Result<(), String> {
    App::run(
        &mut State {
            light_dir: vec3(0., 0., -1.).normalize(),
            spheres: vec![
                Sphere::new(Vec3::new(0., 0., 0.), 0.5, Vec3::new(1., 0., 1.)),
                Sphere::new(Vec3::new(2., 0.5, 0.), 1.5, Vec3::new(0., 0.8, 1.)),
            ],
        },
        render,
    )
}
