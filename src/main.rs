mod camera;
mod hit;
mod material;
mod ray;
mod sphere;
mod vec;

use std::sync::Arc;

use rand::prelude::*;
use rayon::prelude::*;

use camera::Camera;
use hit::{Hit, World, Mesh};
use material::{Metal, Lambertian, Dielectric, Glossy, Emittor};
use ray::Ray;
use sphere::{Sphere, Plane, Triangle};
use vec::{Color, Point3, Vec3};
use indicatif;

fn scene() -> World {
    let mut rng = rand::thread_rng();
    let mut world = World::new();
    let mut mesh = Mesh::new();

    let mat_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let mat_center = Arc::new(Glossy::new(Color::new(0.1, 0.2, 0.5), 0.8));
    let mat_left = Arc::new(Dielectric::new(Color::new(1.0, 0.0, 0.1), 1.5));
    let mat_left_inner = Arc::new(Dielectric::new(Color::new(1.0, 0.0, 0.1), 1.5));
    let mat_right = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 1.0));
    let mat_emit = Arc::new(Emittor::new(Color::new(0.1, 1.0, 1.0)));
    let mat_emit2 = Arc::new(Emittor::new(Color::new(0.4, 0.4, 0.4)));
    let mat_tri = Arc::new(Metal::new(Color::new(1.0, 0.0, 0.0), 0.5));
    
    //let sphere_ground = Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, mat_ground);
    let sphere_center = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, mat_center);
    let sphere_left = Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, mat_left);
    let sphere_left_inner = Sphere::new(Point3::new(-1.0, 0.0, -1.0), -0.45, mat_left_inner);
    let sphere_right = Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, mat_right);
    let sphere_emit = Sphere::new(Point3::new(-1.0, 0.0, -2.0), 0.5, mat_emit);
    let sphere_emit2 = Sphere::new(Point3::new(0.0, 200.0, 0.0), 100.0, mat_emit2);
    let plane = Plane::new(Point3::new(0.0, -0.5, 0.0), Vec3::new(0.0, 1.0, 0.0), mat_ground);

    let vrt0 = Point3::new(0.0, 0.0, -2.0);
    let vrt1 = Point3::new(1.0, 0.0, -2.0);
    let vrt2 = Point3::new(0.0, 1.0, -3.0);
    let tri = Triangle::new(vrt0, vrt1, vrt2, mat_tri.clone());

    let vrt01 = Point3::new(1.0, 1.0, -3.0);
    let vrt11 = Point3::new(1.0, 0.0, -2.0);
    let vrt21 = Point3::new(0.0, 1.0, -3.0);
    let tri1 = Triangle::new(vrt01, vrt11, vrt21, mat_tri);

    mesh.push(tri);
    mesh.push(tri1);
    
    world.push(Box::new(mesh));
    world.push(Box::new(sphere_center));
    world.push(Box::new(sphere_left));
    world.push(Box::new(sphere_left_inner));
    world.push(Box::new(sphere_right));
    world.push(Box::new(sphere_emit));
    world.push(Box::new(sphere_emit2));
    world.push(Box::new(plane));

    world
}

fn ray_color(r: &Ray, world: &World, depth: u64) -> Color {
    if depth <= 0 {
        // If we've exceeded the ray bounce limit, no more light is gathered
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
        if let Some((attenuation, scattered)) = rec.mat.scatter(r, &rec) {
            if rec.mat.emit() {
                attenuation 
            } else {
                attenuation * ray_color(&scattered, world, depth - 1)
            }
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else {
        // Color::new(0.13, 0.28, 0.44)
        // this one gives a dark blue sky

        Color::new(0.0, 0.0, 0.0)
        // this one makes the sky color black
        // so the emittors are the only light sources
    }
}

fn main() {
    // Image
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: u64 = 1980;
    const IMAGE_HEIGHT: u64 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u64;
    const SAMPLES_PER_PIXEL: u64 = 1000;
    const MAX_DEPTH: u64 = 10;

    // World
    let world = scene();


    // Camera
    let cam = Camera::new(Point3::new(-2.0, 2.0, 1.0),
                      Point3::new(0.0, 0.0, -1.0),
                      Vec3::new(0.0, 1.0, 0.0),
                      30.0,
                      ASPECT_RATIO);

    let bar = indicatif::ProgressBar::new(IMAGE_HEIGHT);

    println!("P3");
    println!("{} {}", IMAGE_WIDTH, IMAGE_HEIGHT);
    println!("255");

    for j in (0..IMAGE_HEIGHT).rev() {
        // eprintln!("Scanlines remaining: {}", j + 1);

        bar.inc(1);

        let scanline: Vec<Color> = (0..IMAGE_WIDTH)
            .into_par_iter()
            .map(|i| {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..SAMPLES_PER_PIXEL {
                    let mut rng = rand::thread_rng();
                    let random_u: f64 = rng.gen();
                    let random_v: f64 = rng.gen();

                    let u = ((i as f64) + random_u) / ((IMAGE_WIDTH - 1) as f64);
                    let v = ((j as f64) + random_v) / ((IMAGE_HEIGHT - 1) as f64);

                    let r = cam.get_ray(u, v);
                    pixel_color += ray_color(&r, &world, MAX_DEPTH);
                }

                pixel_color
            })
            .collect();

        for pixel_color in scanline {
            println!("{}", pixel_color.format_color(SAMPLES_PER_PIXEL));
        }
    }

    bar.finish();

    eprintln!("\nDone.");
}