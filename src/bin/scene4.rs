use glam::{vec2, vec3, Vec3};
use ray_tracing::app::App;
use ray_tracing::camera::Camera;
use ray_tracing::light::{Directional, Light, LightSource, Positional};
use ray_tracing::objects::{Intersection, Material, MaterialType, Object3D};
use ray_tracing::renderer::Renderer;
use ray_tracing::scene::Scene;
use ray_tracing::utils::{errors::AppError, image::ImageUtils, plane::Plane, sphere::Sphere};

pub fn update(s: &mut Scene, ts: f32) -> bool {
    let speed = 1.;
    if let Some(Object3D::Sphere(c)) = s.objects.iter_mut().find(|obj| match **obj {
        Object3D::Sphere { .. } => true,
        _ => false,
    }) {
        c.position.y = -ts.sin() * speed + 0.4;
        //c.update();
    };
    true
}
pub fn main() -> Result<(), AppError> {
    let objs = vec![
        Plane::new(vec3(0., 1., 0.), vec3(0., 0., 0.), Some(vec2(5., 5.)), 1),
        Sphere::new(Vec3::new(0., -1., -2.), 1., 1),
    ];

    let mut scene = Scene::new(
        objs,
        vec![
            Material {
                ambience: 0.5,
                diffuse: 0.2,
                shininess: 5.,
                specular: 0.8,
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 1. },
                texture: Some(0),
                ..Default::default()
            },
            Material {
                ambience: 0.4,
                diffuse: 0.4,
                shininess: 64.,
                specular: 0.5,
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 1. },
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
            albedo: vec3(1., 0.8, 0.6),
            position: vec3(1., 3., 2.),
            intensity: 20.,
        }))
        // .with_light(Light::Positional(Positional {
        //     albedo: vec3(1., 0.4, 0.8),
        //     position: vec3(-2., 3., -2.),
        //     intensity: 8.,
        // }))
        // .with_light(Light::Directional(Directional {
        //     albedo: vec3(1., 1., 1.),
        //     direction: vec3(-1., -1., -2.).normalize(),
        //     intensity: 2.,
        // }))
        ;
    //scene1.ambient_color = vec3(0.4, 0.7, 1.);
    scene.update_func = Some(update);
    scene.diffuse = false;
    scene.ray_marching = true;
    scene.enable_accumulation = false;
    scene.shadow_casting = true;

    let mut renderer = Renderer::new();
    let mut camera = Camera::new_with_pos(Vec3::new(0., 2., 4.0), Vec3::new(0., 0., -1.));
    App::run(&mut camera, &mut renderer, &mut scene)
}
