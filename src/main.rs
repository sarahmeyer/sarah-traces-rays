mod camera;
mod hit;
mod material;
mod plane;
mod ray;
mod sphere;
mod vec;

use camera::CameraSettings;
use plane::Plane;
use rand::Rng;
use rayon::prelude::*;
use serde::Deserialize;
use std::io::BufReader;
use std::io::Write;
use std::{env, fs::File};
use vec::Vec3;

use material::{Dielectric, Lambertian, Metal};
use ray::Ray;
use rayon::iter::IntoParallelIterator;
use std::sync::Arc;
use vec::{Color, Point3};

use camera::Camera;
use hit::{Hit, World};
use sphere::Sphere;

#[derive(Deserialize)]
struct Preset {
    // aspect_ratio: f64,
    image_width: u64,
    samples_per_pixel: u64,
    max_depth: u64,
    camera: CameraSettings,
    scene: Option<SceneSettings>,
}

#[derive(Deserialize)]
enum MaterialSettings {
    Dielectric(Dielectric),
    Lambertian(Lambertian),
    Metal(Metal),
}

#[derive(Deserialize)]
struct SceneSettings {
    spheres: Option<Vec<SphereSettings>>,
    planes: Option<Vec<PlaneSettings>>,
}

#[derive(Deserialize)]
struct SphereSettings {
    center: Point3,
    radius: f64,
    // material: MaterialSettings,
}

#[derive(Deserialize)]
struct PlaneSettings {
    point1: Point3,
    point2: Point3,
    normal: Vec3,
    material: MaterialSettings,
}

fn ray_color(r: &Ray, world: &World, depth: u64) -> Color {
    if depth <= 0 {
        // If we've exceeded the ray bounce limit, no more light is gathered
        return Color::new(0.0, 0.0, 0.0);
    }
    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
        if let Some((attenuation, scattered)) = rec.mat.scatter(r, &rec) {
            attenuation * ray_color(&scattered, world, depth - 1)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else {
        let unit_direction = r.direction().normalized();
        let background_color = Color::new(0.6, 0.75, 0.6);
        // let background_color = Color::new(0.5, 0.7, 1.0);
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * background_color
    }
}
fn random_scene() -> World {
    let mut rng = rand::thread_rng();
    let mut world = World::new();

    // let ground_mat = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    let ground_mat = Arc::new(Metal::new(Color::new(0.9, 0.6, 0.5), 0.1));
    let ground_sphere = Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_mat);

    world.push(Box::new(ground_sphere));

    for a in -3..=7 {
        for b in -3..=7 {
            let choose_mat: f64 = rng.gen();
            let center = Point3::new(
                (a as f64) + rng.gen_range(0.0..0.9),
                0.2,
                (b as f64) + rng.gen_range(0.0..0.9),
            );

            if choose_mat < 0.8 {
                // Diffuse
                let albedo = Color::random(0.0..1.0) * Color::random(0.0..1.0);
                let sphere_mat = Arc::new(Lambertian::new(albedo));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            } else if choose_mat < 0.95 {
                // Metal
                let albedo = Color::random(0.4..1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                let sphere_mat = Arc::new(Metal::new(albedo, fuzz));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            } else {
                // Glass
                let sphere_mat = Arc::new(Dielectric::new(1.5));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            }
        }
    }

    // let cube_mat = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));

    // A cube is made of 6 sides. For simplicity, I will make these be perpendicular to the axes.
    // {x, y, z}{min, max} are the bounding box for the rectangular prism that we are going to draw.
    // todo probably should extract this function.
    let xmin = 3.0;
    let xmax = 4.0;
    let ymin = 0.0;
    let ymax = 1.0;
    let zmin = -0.5;
    let zmax = 0.5;

    // let cube_material = Arc::new(Lambertian::new(Color::new(0.2, 0.8, 0.2)));
    let cube_material = Arc::new(Metal::new(Color::new(0.3, 0.2, 0.1), 0.0));

    world.push(Box::new(Plane::new(
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(xmin, ymin, zmin),
        Vec3::new(xmax, ymax, zmin),
        cube_material.clone(),
    )));
    world.push(Box::new(Plane::new(
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(xmin, ymin, zmax),
        Vec3::new(xmax, ymax, zmax),
        cube_material.clone(),
    )));
    world.push(Box::new(Plane::new(
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(xmin, ymin, zmin),
        Vec3::new(xmax, ymin, zmax),
        cube_material.clone(),
    )));
    world.push(Box::new(Plane::new(
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(xmin, ymax, zmin),
        Vec3::new(xmax, ymax, zmax),
        cube_material.clone(),
    )));
    world.push(Box::new(Plane::new(
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(xmin, ymin, zmin),
        Vec3::new(xmin, ymax, zmax),
        cube_material.clone(),
    )));
    world.push(Box::new(Plane::new(
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(xmax, ymin, zmin),
        Vec3::new(xmax, ymax, zmax),
        cube_material.clone(),
    )));

    // let mat1 = Arc::new(Dielectric::new(1.5));
    // let mat2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    // let mat3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));

    // let sphere1 = Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, mat1);
    // let sphere2 = Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, mat2);
    // let sphere3 = Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, mat3);

    // world.push(Box::new(sphere1));
    // world.push(Box::new(sphere2));
    // world.push(Box::new(sphere3));
    // let rect_mat1 = Arc::new(Dielectric::new(1.5));
    // let rect = RectangleXY::new(
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(5.0, 5.0, 5.0),
    //     rect_mat1,
    // );
    // world.push(Box::new(rect));

    // let mat1 = Arc::new(Dielectric::new(1.5));
    // let mat2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    // let mat2_copy = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    // let mat3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));

    // let sphere1 = Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, mat1);
    // let sphere2 = Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, mat2);
    // let sphere2_int = Sphere::new(Point3::new(-4.0, 1.0, 0.0), -0.99, mat2_copy);
    // let sphere3 = Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, mat3);

    // world.push(Box::new(sphere1));
    // world.push(Box::new(sphere2));
    // world.push(Box::new(sphere2_int));
    // world.push(Box::new(sphere3));

    world
}

fn construct_scene_from_settings(scene_settings: &Option<SceneSettings>) -> World {
    if scene_settings.is_some() {
        let scene_settings = scene_settings.as_ref().unwrap();
        let mut world = World::new();
        if scene_settings.spheres.is_some() {
            let sphere_settings = scene_settings.spheres.as_ref().unwrap();
            for sphere_setting in sphere_settings {
                let mat2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
                world.push(Box::new(Sphere::new(
                    sphere_setting.center,
                    sphere_setting.radius,
                    mat2,
                )));
            }
        }
        world
    } else {
        random_scene()
    }
}

fn load_preset_from_file(path_to_file: &str) -> Preset {
    let file = File::open(path_to_file).unwrap();
    let reader = BufReader::new(file);

    let u: Preset = serde_json::from_reader(reader).unwrap();
    u
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let preset: Preset = load_preset_from_file(&args[1]);

    // image
    let image_height: u64 = ((preset.image_width as f64) / preset.camera.aspect_ratio) as u64;

    // World
    // let world = random_scene();
    let world = construct_scene_from_settings(&preset.scene);

    let cam = Camera::new(&preset.camera);

    let mut output = File::create(&args[2]).unwrap();
    writeln!(output, "P3").unwrap();
    writeln!(output, "{} {}", preset.image_width, image_height).unwrap();
    writeln!(output, "255").unwrap();

    for j in (0..image_height).rev() {
        eprintln!("Scanlines remaining: {}", j + 1);

        let scanline: Vec<Color> = (0..preset.image_width)
            .into_par_iter()
            .map(|i| {
                let mut rng = rand::thread_rng();

                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..preset.samples_per_pixel {
                    let random_u: f64 = rng.gen();
                    let random_v: f64 = rng.gen();

                    let u = ((i as f64) + random_u) / ((preset.image_width - 1) as f64);
                    let v = ((j as f64) + random_v) / ((image_height - 1) as f64);

                    let r = cam.get_ray(u, v);
                    pixel_color += ray_color(&r, &world, preset.max_depth);
                }

                pixel_color
            })
            .collect();

        for pixel_color in scanline {
            writeln!(
                output,
                "{}",
                pixel_color.format_color(preset.samples_per_pixel)
            )
            .unwrap();
        }
    }
    eprintln!("Done.");
}
