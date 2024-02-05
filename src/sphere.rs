use std::sync::Arc;

use super::hit::{Hit, HitRecord};
use super::material::Scatter;
use super::ray::Ray;
use super::vec::{Point3, Vec3};

pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Arc<dyn Scatter>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat: Arc<dyn Scatter>) -> Sphere {
        Sphere {
            center,
            radius,
            mat,
        }
    }
}

impl Hit for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = r.origin() - self.center;
        let a = r.direction().length().powi(2);
        let half_b = oc.dot(r.direction());
        let c: f64 = oc.length().powi(2) - self.radius.powi(2);
        let discriminant = half_b.powi(2) - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        let p = r.at(root);
        let mut rec = HitRecord {
            t: root,
            p,
            mat: self.mat.clone(),
            normal: Vec3::new(0.0, 0.0, 0.0),
            front_face: false,
        };

        let outward_normal = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec::Color;

    #[test]
    fn sphere_hits_are_recorded() {
        let mat_diffuse_green = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
        let center = Point3::new(0.0, -100.5, -1.0);
        let sphere = Sphere::new(center, 100.0, mat_diffuse_green);
        // make two vectors, check their intersection points on the sphere
        let u = Ray::new(center, Point3::new(0.0, 0.0, -1.0));
        if let Some(rec) = sphere.hit(&u, 0.001, f64::INFINITY) {
            assert_eq!(rec.p[0], 0.0);
            assert_eq!(rec.p[1], -100.5);
            assert_eq!(rec.p[2], -101.0);
        } else {
            assert!(false);
        }
    }
}
