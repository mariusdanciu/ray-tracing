use std::time::Instant;

use app::App;
use camera::Camera;
use glam::{vec2, vec3, Mat4, Vec2, Vec3};
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

pub fn update(s: &mut Scene, ts: f32) -> bool {
    let speed = 40.;
    if let Some(Object3D::Box {
        position,
        rotation_axis,
        transform,
        inv_transform,
        dimension,
        material_index,
    }) = s.objects.iter_mut().find(|obj| match **obj {
        Object3D::Box { .. } => true,
        _ => false,
    }) {
        rotation_axis.x += 2. * speed * ts;
        rotation_axis.z += 4. * speed * ts;
        rotation_axis.y += 2. * speed * ts;

        let t = Mat4::from_translation(*position)
            * Mat4::from_rotation_x(rotation_axis.x * geometry::DEGREES)
            * Mat4::from_rotation_y(rotation_axis.y * geometry::DEGREES)
            * Mat4::from_rotation_z(rotation_axis.z * geometry::DEGREES);
        transform.x_axis = t.x_axis;
        transform.y_axis = t.y_axis;
        transform.z_axis = t.z_axis;
        transform.w_axis = t.w_axis;

        let inv_t = transform.inverse();
        inv_transform.x_axis = inv_t.x_axis;
        inv_transform.y_axis = inv_t.y_axis;
        inv_transform.z_axis = inv_t.z_axis;
        inv_transform.w_axis = inv_t.w_axis;
    };
    true
}

pub fn main() -> Result<(), AppError> {
    let objs = vec![
        Object3D::new_sphere(Vec3::new(1.2, 0., 2.5), 0.5, 0),
        Object3D::new_sphere(Vec3::new(-1., 0., 0.), 0.5, 2),
        Object3D::new_sphere(Vec3::new(1.5, 0., 0.), 0.5, 4),
        Object3D::new_plane(vec3(0., 1., 0.), vec3(0., -0.5, 0.), 1, Some(vec2(5., 5.))),
        Object3D::new_box(vec3(-1.0, 1., 2.), vec3(0., 0., 0.), vec3(0.5, 1.5, 0.5), 3),
        Object3D::new_cylinder(vec3(3.3, 0.1, 2.1), 1., vec3(90., 0., 0.), 0.4, 5),
    ];

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
                ambience: 0.4,
                diffuse: 0.1,
                shininess: 15.,
                specular: 0.8,
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 0.8 },
                texture: Some(0),
                ..Default::default()
            },
            Material {
                ambience: 0.4,
                diffuse: 2.0,
                shininess: 90.,
                specular: 1.2,
                albedo: Vec3::new(0.0, 0.2, 0.9),
                kind: MaterialType::Reflective { roughness: 1. },
                ..Default::default()
            },
            Material {
                ambience: 0.4,
                shininess: 70.,
                specular: 1.1,
                diffuse: 0.8,
                albedo: Vec3::new(0.5, 0.5, 0.5),
                kind: MaterialType::Reflective { roughness: 0.8 },
                texture: Some(1),
                ..Default::default()
            },
            Material {
                ambience: 0.4,
                diffuse: 0.8,
                shininess: 80.,
                specular: 0.4,
                albedo: Vec3::new(0.8, 0.6, 0.1),
                kind: MaterialType::Reflective { roughness: 0.4 },
                ..Default::default()
            },
            Material {
                ambience: 0.4,
                diffuse: 0.8,
                shininess: 80.,
                specular: 0.4,
                albedo: Vec3::new(0., 0.5, 0.),
                kind: MaterialType::Reflective { roughness: 0.4 },
                ..Default::default()
            },
        ],
    );

    scene1 = scene1
        .with_texture(ImageUtils::load_image("./resources/chess.png")?)
        .with_texture(ImageUtils::load_image("./resources/wood.png")?)
        .with_light(scene::Light::Positional {
            position: vec3(2., 2., 2.),
            intensity: 5.,
        });
    //scene1.ambient_color = vec3(0.4, 0.7, 1.);
    scene1.update_func = Some(update);
    scene1.diffuse = false;
    scene1.enable_accumulation = false;
    scene1.shadow_casting = false;
    scene1.max_frames_rendering = 5000;

    let scene2 = Scene {
        max_ray_bounces: 5,
        max_frames_rendering: 10000,
        light: scene::Light::Directional {
            direction: vec3(1., -1., -1.).normalize(),
            intensity: 2.,
        },
        ambient_color: vec3(0., 0., 0.),
        diffuse: true,
        enable_accumulation: true,
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
        Vec3::new(3.8536084, 0.75215954, 4.388293),
        Vec3::new(-0.76750606, -0.05052291, -0.6390541),
    );
    //let mut camera = Camera::new();
    App::run(&mut camera, &mut renderer)
}
