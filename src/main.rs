use std::time::Instant;

use app::App;
use camera::Camera;
use glam::{vec3, Mat4, Vec3};
use objects::{Material, MaterialType, Object3D};
use scene::Scene;
use utils::{errors::AppError, geometry, image::ImageUtils};

mod app;
mod camera;
mod objects;
mod ray;
mod renderer;
mod scene;
mod utils;

fn translate(p: Vec3) -> Mat4 {
    Mat4::from_translation(p)
}
fn rotate(position: Vec3, time: f32) -> Mat4 {
    //let speed = 0.1;
    //let time = start_time.elapsed().as_millis() as f32;
    //println!("{}", time);

    return translate(position)
        * (geometry::rotate_x_mat(-80. * std::f32::consts::PI / 180.)
            * geometry::rotate_y_mat(-20. * std::f32::consts::PI / 180.));
}
pub fn main() -> Result<(), AppError> {
    let mut objs = vec![
        Object3D::new_sphere(Vec3::new(-1.2, 0., 0.2), 0.5, 0),
        Object3D::new_sphere(Vec3::new(0., 0., 0.), 0.5, 2),
        Object3D::new_sphere(Vec3::new(0., 0.5, -1.), 0.5, 4),
    ];

    objs.push(Object3D::new_triangle(
        Vec3::new(-5.0, -0.5, 5.),
        Vec3::new(5.0, -0.5, 5.),
        Vec3::new(-5., -0.5, -5.),
        1,
    ));
    objs.push(Object3D::new_triangle(
        Vec3::new(5.0, -0.5, -5.),
        Vec3::new(-5.0, -0.5, -5.),
        Vec3::new(5.0, -0.5, 5.),
        1,
    ));

    objs.push(Object3D::new_box(
        vec3(-1.0, 1., 2.),
        vec3(0., 0., 0.),
        vec3(0.5, 1.5, 0.5),
        3,
    ));

    let mut scene1 = Scene::new(
        objs,
        vec![
            Material {
                ambience: 0.4,
                diffuse: 1.3,
                shininess: 3.,
                specular: 5.,
                albedo: Vec3::new(1., 1., 1.),
                kind: MaterialType::Refractive {
                    transparency: 1.,
                    refraction_index: 1.08,
                    reflectivity: 0.1,
                },
                ..Default::default()
            },
            Material {
                ambience: 0.5,
                diffuse: 0.1,
                shininess: 15.,
                specular: 0.8,
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 0.7 },
                texture: Some(0),
                ..Default::default()
            },
            Material {
                ambience: 0.4,
                diffuse: 1.8,
                shininess: 20.,
                specular: 0.9,
                albedo: Vec3::new(0.0, 0.2, 0.9),
                kind: MaterialType::Reflective { roughness: 0.9 },
                ..Default::default()
            },
            Material {
                ambience: 0.4,
                shininess: 30.,
                specular: 1.1,
                diffuse: 0.8,
                albedo: Vec3::new(0.5, 0.5, 0.5),
                kind: MaterialType::Reflective { roughness: 0.8 },
                texture: Some(1),
                ..Default::default()
            },
            Material {
                ambience: 0.2,
                diffuse: 1.2,
                shininess: 90.,
                specular: 0.2,
                albedo: Vec3::new(0.1, 0.6, 0.1),
                kind: MaterialType::Reflective { roughness: 0.3 },
                ..Default::default()
            },
        ],
    );

    scene1 = scene1
        .with_texture(ImageUtils::load_image("./resources/chess.png")?)
        .with_texture(ImageUtils::load_image("./resources/wood.png")?)
        .with_light(scene::Light {
            direction: vec3(-1., -1., -1.).normalize(),
            power: 1.5,
        });
    //scene1.ambient_color = vec3(0.4, 0.7, 1.);
    scene1.difuse = false;
    scene1.shadow_casting = false;
    scene1.max_frames_rendering = 1000;

    let scene2 = Scene {
        max_ray_bounces: 5,
        max_frames_rendering: 5000,
        light: scene::Light {
            direction: vec3(1., -1., -1.).normalize(),
            power: 1.5,
        },
        ambient_color: vec3(0., 0., 0.),
        difuse: true,
        objects: vec![
            Object3D::new_sphere(Vec3::new(0., -100.5, 0.), 100., 0),
            Object3D::new_sphere(Vec3::new(10., 15., -34.), 20.0, 1),
            Object3D::new_sphere(Vec3::new(0., 0.5, -0.5), 1., 2),
        ],
        materials: vec![
            Material {
                albedo: Vec3::new(0.9, 0.9, 0.2),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 0.0,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.9, 0.5, 0.2),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 8.0,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.9, 0.9, 0.2),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 0.,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let mut renderer = renderer::Renderer::new(scene1);
    let mut camera = Camera::new_with_pos(
        Vec3::new(-2.8777819, 1.3294921, 2.0364523),
        Vec3::new(0.6106094, -0.19236837, -0.76821935),
    );
    //let mut camera = Camera::new();
    App::run(&mut camera, &mut renderer)
}
