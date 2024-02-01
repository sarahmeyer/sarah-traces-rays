use std::sync::Arc;

use crate::{
    hit::{Hit, HitRecord},
    material::Scatter,
    ray::Ray,
    vec::{Point3, Vec3},
};

pub struct RectangleXY {
    min_point: Point3,
    max_point: Point3,
    mat: Arc<dyn Scatter>,
}

impl RectangleXY {
    pub fn new(min_point: Point3, max_point: Point3, mat: Arc<dyn Scatter>) -> RectangleXY {
        RectangleXY {
            min_point,
            max_point,
            mat,
        }
    }
}

impl Hit for RectangleXY {
    fn hit(&self, r: &Ray, _t_min: f64, _t_max: f64) -> Option<HitRecord> {
        let z = self.min_point.z();

        let t = (z - r.origin().z()) / r.direction().z();
        let p = r.origin() + r.direction() * t;

        if p.x() <= self.max_point.x()
            && p.x() >= self.min_point.x()
            && p.y() <= self.max_point.y()
            && p.y() >= self.min_point.y()
        {
            let rec = HitRecord {
                t,
                p,
                mat: self.mat.clone(),
                normal: Vec3::new(0.0, 0.0, 0.0),
                front_face: false,
            };
            // todo: always front face?
            Some(rec)
        } else {
            return None;
        }
    }
}
