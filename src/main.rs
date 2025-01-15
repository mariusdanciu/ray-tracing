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
        center: Vec3::new(-0.9, 0.3, -1.3),
        length: 1.0,
        width: 1.,
        depth: 1.,
    };

    let mut objs = vec![
        Object3D::new_sphere(Vec3::new(-0.9, 1., -1.3), 0.2, 2),
        Object3D::new_sphere(Vec3::new(-0.6, -0.0, -0.2), 0.5, 0),
        //Object3D::new_sphere(Vec3::new(0., -100.5, 0.), 100., 2),
    ];

    objs.append(&mut cube.triangles(3));
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

    let mut scene1 = Scene::new(
        objs,
        
        vec![
            Material {
                albedo: Vec3::new(1., 1., 1.),
                kind: MaterialType::Refractive {
                    transparency: 0.8,
                    refraction_index: 0.9,
                },
                //kind: MaterialType::Reflective { roughness: 1.0 },
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 0.2 },
                texture: Some(0),
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.2, 0.5, 1.0),
                kind: MaterialType::Reflective { roughness: 0.4 },
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 0.1 },
                ..Default::default()
            },
        ],
    );
    scene1 = scene1.with_texture(ImageUtils::load_image("./resources/chess.png")?);
    scene1.difuse = false;
    scene1.max_frames_rendering = 20000;
    scene1.ambient_color = vec3(0., 0., 0.);

    let scene2 = Scene {
        max_ray_bounces: 5,
        max_frames_rendering: 5000,
        light_dir: vec3(1., -1., -1.).normalize(),
        ambient_color: vec3(0.0, 0.0, 0.0),
        difuse: true,
        objects: vec![
            Object3D::new_sphere(Vec3::new(0., 0., -0.5), 0.5, 0),
            Object3D::new_sphere(Vec3::new(0., -100.5, 0.), 100., 1),
            //Sphere::new(Vec3::new(0.5, 0.0, 1.0), 0.5, 2),
            Object3D::new_sphere(Vec3::new(10., 5., -24.), 10.0, 3),
        ],
        materials: vec![
            Material {
                albedo: Vec3::new(0.3, 0.0, 1.0),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 0.,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.9, 0.9, 0.2),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 0.0,
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
                emission_power: 10.0,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 0.2 },
                emission_power: 0.8,
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
    App::run(&mut camera, &mut renderer)
}
