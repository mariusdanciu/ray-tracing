

use ray_tracing::app::App;
use ray_tracing::camera::Camera;
use glam::{vec2, vec3,  Vec3};
use ray_tracing::objects::{Material, MaterialType, Object3D};
use ray_tracing::scene::{Scene, Light};
use ray_tracing::renderer::Renderer;
use ray_tracing::utils::{
    cone::Cone, cuboid::Cuboid, cylinder::Cylinder, errors::AppError, image::ImageUtils,
    plane::Plane, sphere::Sphere, triangle::Triangle,
};


pub fn update(s: &mut Scene, ts: f32) -> bool {
    let speed = 0.2;
    if let Some(Object3D::Cuboid(c)) = s.objects.iter_mut().find(|obj| match **obj {
        Object3D::Cuboid { .. } => true,
        _ => false,
    }) {
        c.rotation_axis.x += 2. * speed;
        c.rotation_axis.z += 4. * speed;
        c.rotation_axis.y += 2. * speed;
        c.update();
    };
    true
}

pub fn main() -> Result<(), AppError> {

    let scene2 = Scene {
        max_ray_bounces: 5,
        max_frames_rendering: 10000,
        ambient_color: vec3(0., 0., 0.),
        diffuse: true,
        enable_accumulation: true,
        objects: vec![
            Sphere::new(Vec3::new(0., -100.5, 0.), 100., 0),
            Sphere::new(Vec3::new(10., 15., -34.), 20.0, 1),
            Sphere::new(Vec3::new(0., 0.5, -0.5), 1., 2),
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

    let mut renderer = Renderer::new(scene2);
    let mut camera = Camera::new_with_pos(
        Vec3::new(3.8536084, 0.75215954, 4.388293),
        Vec3::new(-0.76750606, -0.05052291, -0.6390541),
    );
    //let mut camera = Camera::new();
    App::run(&mut camera, &mut renderer)
}
