use glam::{vec3, Vec3};
use ray_tracing::app::App3D;
use ray_tracing::camera::Camera;
use ray_tracing::objects::{Material, MaterialType, Object3D};
use ray_tracing::renderer::Renderer;
use ray_tracing::scene::Scene;
use ray_tracing::utils::{errors::AppError, sphere::Sphere};

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
    let mut scene = Scene {
        max_ray_bounces: 5,
        ambient_color: vec3(0., 0., 0.),
        diffuse: true,
        objects: vec![
            Sphere::new(Vec3::new(0., -100.5, 0.), 100., 0),
            Sphere::new(Vec3::new(10., 15., -40.), 20.0, 1),
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
                emission_power: 14.0,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.0, 0.3, 0.7),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 0.8,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let mut renderer = Renderer::new();
    renderer.enable_accumulation = true;

    let mut camera = Camera::new_with_pos(
        Vec3::new(3.8536084, 0.75215954, 4.388293),
        Vec3::new(-0.76750606, -0.05052291, -0.6390541),
    );
    //let mut camera = Camera::new();

    App3D::run(&mut camera, &mut scene, &mut renderer)
}
