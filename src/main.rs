use app::App;
use camera::Camera;
use glam::{vec3, Vec3};
use objects::{Cuboid, Material, MaterialType, Object3D};
use scene::Scene;
use utils::{errors::AppError, image::ImageUtils};

mod app;
mod camera;
mod objects;
mod ray;
mod renderer;
mod scene;
mod utils;

pub fn main() -> Result<(), AppError> {
    let cube = Cuboid {
        center: Vec3::new(-0.4, 0., -1.3),
        length: 1.0,
        width: 1.,
        depth: 1.,
    };

    let mut objs = vec![
        Object3D::new_sphere(Vec3::new(-1.2, 0., 0.2), 0.5, 0),
        Object3D::new_sphere(Vec3::new(0., 0.5, 0.), 0.5, 2),
    ];

   // objs.append(&mut cube.triangles(3));

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
        vec3(0.0, 0.1, -1.0),
        vec3(1.5, 0.5, 0.1),
        3,
    ));

    let mut scene1 = Scene::new(
        objs,
        vec![
            Material {
                ambience: 0.3,
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
                ambience: 0.3,
                diffuse: 0.1,
                shininess: 15.,
                specular: 0.8,
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 0.4 },
                texture: Some(0),
                ..Default::default()
            },
            Material {
                ambience: 0.2,
                diffuse: 1.8,
                shininess: 20.,
                specular: 1.9,
                albedo: Vec3::new(0.0, 0.2, 0.9),
                kind: MaterialType::Reflective { roughness: 0.2 },
                ..Default::default()
            },
            Material {
                shininess: 10.,
                specular: 1.,
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 0.2 },
                ..Default::default()
            },
        ],
    );

    scene1 = scene1
        .with_texture(ImageUtils::load_image("./resources/chess.png")?)
        .with_light(scene::Light {
            direction: vec3(-1., -1., -1.).normalize(),
            power: 1.5,
        });
    scene1.difuse = false;
    scene1.shadow_casting = true;
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
