use std::sync::Arc;

use crate::{
    hit::{Hit, HitRecord},
    material::Scatter,
    ray::Ray,
    vec::{Point3, Vec3},
};

fn scalar(p: Point3) -> f64 {
    p.x() + p.y() + p.z()
}

// TODO: in the future, we probably want to be drawing triangles rather than (rectangular) plane segments.
// This is kind of a hack, but for now, define a plane segment as (a) the normal vector, and (b) two points that make up its diagonal.
pub struct Plane {
    normal: Vec3,
    point1: Point3,
    point2: Point3,
    mat: Arc<dyn Scatter>,
}

impl Plane {
    pub fn new(normal: Point3, point1: Point3, point2: Point3, mat: Arc<dyn Scatter>) -> Plane {
        // check both points on the same plane
        let p1 = normal * point1;
        let p2 = normal * point2;
        assert!(
            scalar(p1) == scalar(p2),
            "The two points provided need to fall on the same plane!"
        );

        Plane {
            normal,
            point1,
            point2,
            mat,
        }
    }
}

fn between(x1: f64, x2: f64, p: f64) -> bool {
    let eps = 0.01;
    if x1 < x2 {
        x1 <= p + eps && p - eps <= x2
    } else {
        x2 <= p + eps && p - eps <= x1
    }
}

impl Hit for Plane {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // To find out where the ray intersects our plane, we just need to write out the ray and plane equations and set them equal.
        // See https://brilliant.org/wiki/3d-coordinate-geometry-equation-of-a-plane/ for the
        // equation of a plane, as determined by its normal and one point that intersects it.
        //
        // my derivation steps below:
        //       0 = scalar(self.normal * (p - self.point))                                          // where p is the intersection point
        //  =>   0 = scalar(self.normal * ((root * r.direction() + r.origin()) - self.point))        // p is parametrized by root, aka t.
        //  =>   0 = scalar(self.normal * root * r.direction() + self.normal * r.origin() - self.normal * self.point)          // expand out self.normal
        //  =>   0 = root * scalar(self.normal * r.direction()) + scalar(self.normal * r.origin() - self.normal * self.point)  // take root out of scalar
        //  =>   root = -scalar(self.normal * r.origin() - self.normal * self.point) / scalar(self.normal * r.direction())     // rearrange to solve for root

        let frac_top = -scalar(self.normal * r.origin() - self.normal * self.point1);
        let frac_bottom = scalar(self.normal * r.direction());
        let root = frac_top / frac_bottom;

        // Discard bad solutions
        if frac_bottom == 0.0 {
            return None;
        }
        if root < t_min || root > t_max {
            return None;
        }

        let p = r.at(root);

        let rec = HitRecord {
            t: root,
            p,
            mat: self.mat.clone(),
            normal: self.normal,
            front_face: false,
        };

        // We don't want the entire plane, only a plane segment between point1 and point2.
        if between(self.point1.x(), self.point2.x(), p.x())
            && between(self.point1.y(), self.point2.y(), p.y())
            && between(self.point1.z(), self.point2.z(), p.z())
        {
            return Some(rec);
        } else {
            return None;
        }
    }
}
