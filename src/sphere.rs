use std::sync::Arc;

use super::vec::{Vec3, Point3};
use super::ray::Ray;
use super::hit::{Hit, HitRecord};
use super::material::Scatter;

pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Arc<dyn Scatter>
}

impl Sphere {
    pub fn new(cen: Point3, r: f64, m: Arc<dyn Scatter>) -> Sphere {
        Sphere {
            center: cen,
            radius: r,
            mat: m
        }
    }
}

impl Hit for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = r.origin() - self.center;
        let a = r.direction().length().powi(2);
        let half_b = oc.dot(r.direction());
        let c = oc.length().powi(2) - self.radius.powi(2);
        
        let discriminant = half_b.powi(2) - a * c;
        if discriminant < 0.0 {
            return None;
        }

        // Find the nearest root that lies in the acceptable range
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let mut rec = HitRecord {
            t: root,
            p: r.at(root),
            mat: self.mat.clone(),
            normal: Vec3::new(0.0, 0.0, 0.0),
            front_face: false
        };

        let outward_normal = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}



pub struct Plane {
    center: Point3,
    normal: Vec3,
    mat: Arc<dyn Scatter>
}

impl Plane {
    pub fn new(c: Point3, n: Vec3, m: Arc<dyn Scatter>) -> Plane {
        Plane {
            center: c,
            normal: n,
            mat: m
        }
    }
}

impl Hit for Plane {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let denom = self.normal.dot(r.direction());
        //eprintln!("{}", denom);
        let solution = if denom.abs() > 1e-6 {
            let p0l0 = self.center - r.origin();
            let t = p0l0.dot(self.normal) / denom;
            if t >= t_min && t <= t_max {
                //eprintln!("collision!");
                Some(t)
            } else {
                //eprintln!("no hit because too close");
                None
            }
        } else {
            //eprintln!("no hit because denom too small");
            None
        };

        if let Some(root) = solution {
            let mut rec = HitRecord {
                t: root,
                p: r.at(root),
                mat: self.mat.clone(),
                normal: Vec3::new(0.0, 0.0, 0.0),
                front_face: false
            };

            rec.set_face_normal(r, self.normal);
            //eprintln!("hit a plane!");

            return Some(rec);
        } else {
            return None;
        }
    }
}



pub struct Triangle {
    v0: Point3,
    v1: Point3,
    v2: Point3,
    mat: Arc<dyn Scatter>
}

impl Triangle {
    pub fn new(vert0: Point3, vert1: Point3, vert2: Point3, mater: Arc<dyn Scatter>) -> Triangle {
        Triangle {
            v0: vert0,
            v1: vert1,
            v2: vert2,
            mat: mater
        }
    }
}

impl Hit for Triangle {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let edge0 = self.v1 - self.v0;        
        let edge1 = self.v2 - self.v1;
        let edge2 = self.v0 - self.v2;
        let norm = edge0.cross(edge1);
        let dist = (-1.0) * norm.dot(self.v0);
        let root = (-1.0) * (norm.dot(r.origin()) + dist) / norm.dot(r.direction());

        let phit = r.at(root);

        if norm.dot(r.direction()).abs() < 1e-6 || root < 0.0 {
            return None;
        }

        let vp0 = phit - self.v0;
        let c0 = edge0.cross(vp0);
        let vp1 = phit - self.v1;
        let c1 = edge1.cross(vp1);
        let vp2 = phit - self.v2;
        let c2 = edge2.cross(vp2);

        if norm.dot(c0) < t_min || norm.dot(c1) < t_min || norm.dot(c2) < t_min || 
            norm.dot(c0) > t_max || norm.dot(c1) > t_max || norm.dot(c2) > t_max {
            return None;
        } else {
            let mut rec = HitRecord {
                t: root,
                p: r.at(root),
                mat: self.mat.clone(),
                normal: Vec3::new(0.0, 0.0, 0.0),
                front_face: false
            };
        
            rec.set_face_normal(r, norm);
            //eprintln!("hit a plane!");
        
            return Some(rec);
        }
    }
}