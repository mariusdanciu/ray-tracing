use std::sync::Arc;

use app::App;
use glam::{vec3, Vec3};
use scene::{Material, MaterialType, Object3D, Scene};

mod app;
mod camera;
mod scene;
mod renderer;
mod ray;

pub fn main() -> Result<(), String> {

    let scene1 = Scene {
        max_ray_bounces: 5,
        light_dir: vec3(-1., -1., -1.).normalize(),
        ambient_color: vec3(0., 0.0, 0.0),
        difuse: false,
        objects: vec![
            Object3D::new_sphere(Vec3::new(-0.3, 0.0, -0.5), 0.5, 0),
            Object3D::new_sphere(Vec3::new(0., -100.5, 0.), 100., 2),
            Object3D::new_triangle(vec3(-1.5, 0.5, 0.0), vec3(-1.5, -0.5, 0.0), vec3(-0.5, -0.5, -1.5), 1),
            Object3D::new_triangle(vec3(-1.5, 0.5, 0.0), vec3(-0.5, -0.5, -1.5), vec3(-0.5, 0.5, -1.5), 1),
        ],
        materials: vec![
            Material {
                albedo: Vec3::new(0.9, 0.1, 0.0),
                kind: MaterialType::Reflective { roughness: 1.0 },
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 0.1 },
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.1, 0.4, 1.0),
                kind: MaterialType::Reflective { roughness: 0.1 },
                ..Default::default()
            },
        ],
    };

    let _scene2 = Scene {
        max_ray_bounces: 5,
        light_dir: vec3(1., -1., -1.).normalize(),
        ambient_color: vec3(0.0, 0.0, 0.0),
        difuse: true,
        objects: vec![
            Object3D::new_sphere(Vec3::new(0., 0., -0.5), 0.5, 0),
            Object3D::new_sphere(Vec3::new(0., -100.5, 0.), 100., 1),
            //Sphere::new(Vec3::new(0.5, 0.0, 1.0), 0.5, 2),
            Object3D::new_sphere(Vec3::new(10., 5., -24.), 10.0, 3),
            Object3D::new_triangle(vec3(-1.5, 0.5, 0.0), vec3(-1.5, -0.5, 0.0), vec3(-0.5, -0.5, -1.5), 4),
            Object3D::new_triangle(vec3(-1.5, 0.5, 0.0), vec3(-0.5, -0.5, -1.5), vec3(-0.5, 0.5, -1.5), 4),
        ],
        materials: vec![
            Material {
                albedo: Vec3::new(0.3, 0.0, 1.0),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 1.8,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.9, 0.9, 0.2),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 0.2,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(1.0, 1.0, 1.0),
                kind: MaterialType::Refractive {
                    transparency: 1.0,
                    refraction_index: 0.97,
                },
                emission_power: 0.0,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.8, 0.5, 0.2),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 26.0,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.1, 0.4, 1.0),
                kind: MaterialType::Reflective { roughness: 0.2 },
                emission_power: 1.,
                ..Default::default()
            },
        ],
    };

    let mut renderer = renderer::Renderer::new(Arc::new(scene1));
    App::run(&mut renderer)
}
