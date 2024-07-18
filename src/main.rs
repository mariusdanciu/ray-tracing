extern crate sdl2;

use std::borrow::BorrowMut;
use std::thread::Thread;
use std::time::{Duration, Instant};

use app::App;
use camera::{Camera, CameraEvent};
use fontdue_sdl2::fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use fontdue_sdl2::fontdue::Font;
use fontdue_sdl2::FontTexture;
use glam::{vec3, Vec3};
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

fn to_rgba(c: Vec3) -> (u8, u8, u8, u8) {
    (
        (c.x * 255.) as u8,
        (c.y * 255.) as u8,
        (c.z * 255.) as u8,
        (255.) as u8,
    )
}

fn pixel(
    ray_orig: Vec3,
    ray_dir: Vec3,
    light_dir: Vec3,
    state: &mut State,
) -> (u8, u8, u8, u8) {
    // (bx^2 + by^2)t^2 + (2(axbx + ayby))t + (ax^2 + ay^2 - r^2) = 0
    // where
    // a = ray origin
    // b = ray direction
    // r = radius
    // t = hit distance

    let mut closest_sphere = &state.spheres[0];
    let mut closest_t = -100.0;
    for sphere in state.spheres.iter() {

        let origin = ray_orig - sphere.position;

        let a = ray_dir.dot(ray_dir);
        let b = 2. * origin.dot(ray_dir);
        let c = origin.dot(origin) - sphere.radius * sphere.radius;

        let disc = b * b - 4. * a * c;

        if disc < 0.0 {
            continue;
        }

        // closest to ray origin
        let t = (-b + disc.sqrt()) / (2.0 * a);
        //let t1 = (-b + disc.sqrt()) / 2.0 * a;

        if t > closest_t {
            closest_t = t;
            closest_sphere = sphere;
        }


    }

    if closest_t == -100. {
        return (0, 0, 0, 0);
    }

    let origin = ray_orig - closest_sphere.position;
    let hit_point = origin + (ray_dir) * closest_t;

    let normal = hit_point.normalize();

    let mut d = normal.dot(-light_dir);
    if d < 0.0 {
        d = 0.0;
    }

    let color = closest_sphere.color * d;

    to_rgba(color)
}

fn draw(
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

    let sphere_color = Vec3::new(1., 0., 1.);

    texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
        //let time = Instant::now();

        let mut i = 0;

        for ray_dir in camera.ray_directions.iter() {
            let color = pixel(
                camera.position,
                *ray_dir,
                state.light_dir,
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
            light_dir: vec3(-1., -2., -5.).normalize(),
            spheres: vec![
                Sphere::new(Vec3::new(0., 0., 0.), 0.5, Vec3::new(1., 0., 1.)),
                Sphere::new(Vec3::new(1., 0., -3.), 1.7, Vec3::new(0., 1., 0.))
            ],
        },
        draw,
    )
}
