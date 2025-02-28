
use glam::{vec2, vec3, Vec3};
use ray_tracing::app::App3D;
use ray_tracing::camera::Camera;
use ray_tracing::light::{Directional, Light, Positional};
use ray_tracing::objects::{Material, MaterialType, Object3D};
use ray_tracing::renderer::Renderer;
use ray_tracing::scene::Scene;
use ray_tracing::utils::cuboid::Cuboid;
use ray_tracing::utils::cylinder::Cylinder;
use ray_tracing::utils::substraction::Substraction;
use ray_tracing::utils::union::Union;
use ray_tracing::utils::{errors::AppError, image::ImageUtils, plane::Plane, sphere::Sphere};

pub fn update(s: &mut Scene, ts: f32) -> bool {
    let speed = 1.;

    if let Some(Object3D::Sphere( c))= s.objects.get_mut(2) {
        c.position.y = (ts).sin() * speed + 0.8;
        c.update();
    };
    if let Some(Object3D::Cylinder( cy))= s.objects.get_mut(3) {

        cy.rotation_axis.y += 2. * speed;
        cy.update();
    };
    true
}


pub fn main() -> Result<(), AppError> {
    let objs = vec![
        Union::new(1, 2),
        Plane::new(vec3(0., 1., 0.), vec3(0., 0., 0.), Some(vec2(5., 5.)), 0),
        Sphere::new(Vec3::new(0., -1., -2.), 1., 1),
        Cylinder::new(vec3(-1., 1.2, 0.2), 0.5, vec3(0., 0., 45.), 1.5, 2),
        Cuboid::new(vec3(-1., 1.5, 0.2), vec3(0., 20., 0.), vec3(0.5, 1., 0.5), 1),
        Substraction::new(3, 4)
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
                ambience: 0.3,
                diffuse: 0.4,
                shininess: 64.,
                specular: 0.5,
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 1. },
                ..Default::default()
            },
            Material {
                ambience: 0.4,
                diffuse: 0.4,
                shininess: 64.,
                specular: 0.5,
                albedo: Vec3::new(0.0, 0.4, 1.),
                kind: MaterialType::Reflective { roughness: 1. },
                ..Default::default()
            },
        ]
    );

    scene.sdfs = vec!(0, 3);

    scene = scene
        .with_texture(ImageUtils::load_image("./resources/chess.png")?)
        .with_texture(ImageUtils::load_image("./resources/wood.png")?)
        .with_texture(ImageUtils::load_image("./resources/stone3.jpg")?)
        .with_texture(ImageUtils::load_image("./resources/earth_clouds.jpg")?)
        // .with_light(Light::Positional(Positional {
        //     albedo: vec3(1., 0.8, 0.6),
        //     position: vec3(1., 3., 2.),
        //     intensity: 20.,
        // }))
        // .with_light(Light::Positional(Positional {
        //     albedo: vec3(1., 0.4, 0.8),
        //     position: vec3(-2., 3., -2.),
        //     intensity: 8.,
        // }))
        .with_light(Light::Directional(Directional {
            albedo: vec3(1., 1., 1.),
            direction: vec3(-1., -1., -2.).normalize(),
            intensity: 2.,
        }))
        ;
    //scene1.ambient_color = vec3(0.4, 0.7, 1.);
    scene.update_func = Some(update);
    scene.diffuse = false;
    scene.ray_marching = true;
    scene.shadow_casting = true;

    let mut renderer = Renderer::new();
    let mut camera = Camera::new_with_pos(Vec3::new(0., 2., 4.0), Vec3::new(0., 0., -1.));
    
    App3D::run(&mut camera, &mut scene, &mut renderer)
}
