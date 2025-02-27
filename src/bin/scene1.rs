use glam::{vec2, vec3, Vec3};
use ray_tracing::app::App3D;
use ray_tracing::camera::Camera;
use ray_tracing::light::{Light, Positional};
use ray_tracing::objects::{Material, MaterialType, Object3D};
use ray_tracing::renderer::Renderer;
use ray_tracing::scene::Scene;
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
    let objs = vec![
        Sphere::new(Vec3::new(1.2, 0., 2.5), 0.5, 0),
        Plane::new(vec3(0., 1., 0.), vec3(0., -0.5, 0.), Some(vec2(5., 5.)), 1),
        Sphere::new_sphere_with_rotation(Vec3::new(3.0, 0.5, 0.8), vec3(-90., 0., 0.), 0.7, 2),
        Cuboid::new(vec3(-1.0, 1.3, 2.), vec3(0., 0., 0.), vec3(0.6, 1., 0.2), 3),
        Sphere::new(Vec3::new(1.5, 0., 0.), 0.5, 4),
        Cone::new(vec3(2.3, 0.7, 2.), 0.5, 1., vec3(120., 0., 0.), 5),
        Cylinder::new(vec3(2.3, 0., 3.0), 1., vec3(90., 0., 0.), 0.4, 6),
        Triangle::new(
            Vec3::new(1.5, 1., 0.),
            Vec3::new(1.5, 0., 0.),
            Vec3::new(2.5, 1., 0.),
            1,
        ),
    ];

    let mut scene = Scene::new(
        objs,
        vec![
            Material {
                ambience: 0.4,
                diffuse: 0.3,
                shininess: 12.,
                specular: 3.,
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
                ambience: 0.2,
                diffuse: 0.8,
                shininess: 200.,
                specular: 1.2,
                albedo: Vec3::new(0.0, 0.2, 0.9),
                texture: Some(3),
                kind: MaterialType::Reflective { roughness: 0.6 },
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
                ambience: 0.5,
                diffuse: 0.1,
                shininess: 80.,
                specular: 0.1,
                albedo: Vec3::new(0.3, 0.7, 0.5),
                kind: MaterialType::Reflective { roughness: 0.4 },
                texture: Some(0),
                ..Default::default()
            },
            Material {
                ambience: 0.6,
                diffuse: 0.3,
                shininess: 40.,
                specular: 0.8,
                albedo: Vec3::new(0.1, 0.5, 0.9),
                kind: MaterialType::Reflective { roughness: 0.4 },
                ..Default::default()
            },
        ],
    );

    scene = scene
        .with_texture(ImageUtils::load_image("./resources/chess.png")?)
        .with_texture(ImageUtils::load_image("./resources/wood.png")?)
        .with_texture(ImageUtils::load_image("./resources/stone3.jpg")?)
        .with_texture(ImageUtils::load_image("./resources/earth_clouds.jpg")?)
        .with_light(Light::Positional(Positional {
            albedo: vec3(1., 1., 1.),
            position: vec3(2., 2., 2.),
            intensity: 6.,
        }))
        .with_light(Light::Positional(Positional {
            albedo: vec3(1., 1., 1.),
            position: vec3(3., 2., -2.),
            intensity: 6.,
        }));
    //scene1.ambient_color = vec3(0.4, 0.7, 1.);
    scene.update_func = Some(update);
    scene.diffuse = false;
    scene.shadow_casting = false;

    let scene2 = Scene {
        max_ray_bounces: 5,
        ambient_color: vec3(0., 0., 0.),
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

    let mut renderer = Renderer::new();
    let mut camera = Camera::new_with_pos(
        Vec3::new(3.8536084, 0.75215954, 4.388293),
        Vec3::new(-0.76750606, -0.05052291, -0.6390541),
    );
    //let mut camera = Camera::new();

    App3D::run(&mut camera, &mut scene, &mut renderer)
}
