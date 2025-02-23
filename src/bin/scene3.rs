use glam::{vec2, vec3, Vec3};
use ray_tracing::app::App;
use ray_tracing::camera::Camera;
use ray_tracing::light::{SphericalPositional, Light, LightSource};
use ray_tracing::objects::{Intersection, Material, MaterialType};
use ray_tracing::renderer::Renderer;
use ray_tracing::scene::Scene;
use ray_tracing::utils::{errors::AppError, image::ImageUtils, plane::Plane, sphere::Sphere};

pub fn update(s: &mut Scene, ts: f32) -> bool {
    false
}

#[derive(Clone)]
struct S<'a> {
    f: Vec<&'a dyn Intersection>,
}

pub fn main() -> Result<(), AppError> {
    let objs = vec![
        Plane::new(vec3(0., 1., 0.), vec3(0., 0., 0.), Some(vec2(5., 5.)), 0),
        Sphere::new(Vec3::new(0., 0.5, 0.), 0.5, 1),
    ];

    let mut scene = Scene::new(
        objs,
        vec![
            Material {
                ambience: 1.6,
                diffuse: 0.2,
                shininess: 5.,
                specular: 0.8,
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 1. },
                //texture: Some(0),
                ..Default::default()
            },
            Material {
                ambience: 0.4,
                diffuse: 0.7,
                shininess: 60.,
                specular: 0.5,
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
        .with_light(Light::SphericalPositional(SphericalPositional {
            albedo: vec3(1., 0.5, 1.),
            position: vec3(1., 3., 2.),
            intensity: 8.,
            radius: 1.,
        }));
    //scene1.ambient_color = vec3(0.4, 0.7, 1.);
    scene.update_func = Some(update);
    scene.diffuse = false;
    scene.enable_accumulation = false;
    scene.shadow_casting = true;

    let mut renderer = Renderer::new();
    let mut camera = Camera::new_with_pos(Vec3::new(0., 2., 5.0), Vec3::new(0., 0., -5.));
    App::run(&mut camera, &mut renderer, &mut scene)
}
