use crate::world::Vec2;

/// Check if a line segment (from `p1` to `p2`) intersects a circle at `center` with given `radius`.
///
/// Uses the closest-point-on-segment approach.
pub fn line_circle_intersect(p1: Vec2, p2: Vec2, center: Vec2, radius: f64) -> bool {
    let d = p2 - p1;
    let f = p1 - center;

    let seg_len_sq = d.dot(d);
    if seg_len_sq < 1e-12 {
        // Degenerate segment (point).
        return f.length() < radius;
    }

    // Parameter t for closest point on the line segment.
    let t = (-f.dot(d) / seg_len_sq).clamp(0.0, 1.0);
    let closest = Vec2::new(p1.x + t * d.x, p1.y + t * d.y);
    let dist = closest.distance_to(center);
    dist < radius
}

/// Ray-line-segment intersection.
///
/// Returns the distance along the ray if it intersects the line segment, or `None`.
/// Ray starts at `origin` going in direction `angle` (radians).
pub fn ray_line_intersect(origin: Vec2, angle: f64, p1: Vec2, p2: Vec2) -> Option<f64> {
    let dir = Vec2::new(angle.cos(), angle.sin());
    let seg = p2 - p1;
    let denom = dir.x * seg.y - dir.y * seg.x;

    if denom.abs() < 1e-12 {
        // Ray and segment are parallel.
        return None;
    }

    let diff = p1 - origin;
    let t = (diff.x * seg.y - diff.y * seg.x) / denom;
    let u = (diff.x * dir.y - diff.y * dir.x) / denom;

    if t >= 0.0 && (0.0..=1.0).contains(&u) {
        Some(t)
    } else {
        None
    }
}

/// Ray-circle intersection.
///
/// Returns the distance to the nearest intersection point of a ray with a circle,
/// or `None` if no intersection.
pub fn ray_circle_intersect(origin: Vec2, angle: f64, center: Vec2, radius: f64) -> Option<f64> {
    let dir = Vec2::new(angle.cos(), angle.sin());
    let oc = origin - center;

    let a = dir.dot(dir); // Should be 1.0 for unit vector, but keep general.
    let b = 2.0 * oc.dot(dir);
    let c = oc.dot(oc) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_disc = discriminant.sqrt();
    let t1 = (-b - sqrt_disc) / (2.0 * a);
    let t2 = (-b + sqrt_disc) / (2.0 * a);

    // Return the nearest positive intersection.
    if t1 > 1e-6 {
        Some(t1)
    } else if t2 > 1e-6 {
        Some(t2)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_2;

    #[test]
    fn line_circle_intersect_touching() {
        // Horizontal line at y=0, circle at (0.5, 0.2) with r=0.3
        assert!(line_circle_intersect(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.5, 0.2),
            0.3,
        ));
    }

    #[test]
    fn line_circle_no_intersect() {
        // Horizontal line at y=0, circle at (0.5, 2.0) with r=0.3
        assert!(!line_circle_intersect(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.5, 2.0),
            0.3,
        ));
    }

    #[test]
    fn line_circle_endpoint() {
        // Circle near the start of the segment
        assert!(line_circle_intersect(
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(0.0, 0.1),
            0.2,
        ));
    }

    #[test]
    fn ray_line_hit() {
        // Ray from (0,0) going right, segment from (5,-1) to (5,1)
        let d = ray_line_intersect(
            Vec2::new(0.0, 0.0),
            0.0,
            Vec2::new(5.0, -1.0),
            Vec2::new(5.0, 1.0),
        );
        assert!(d.is_some());
        assert!((d.unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn ray_line_miss() {
        // Ray from (0,0) going right, segment far above
        let d = ray_line_intersect(
            Vec2::new(0.0, 0.0),
            0.0,
            Vec2::new(5.0, 10.0),
            Vec2::new(5.0, 11.0),
        );
        assert!(d.is_none());
    }

    #[test]
    fn ray_line_behind() {
        // Ray from (0,0) going right, segment behind at x=-5
        let d = ray_line_intersect(
            Vec2::new(0.0, 0.0),
            0.0,
            Vec2::new(-5.0, -1.0),
            Vec2::new(-5.0, 1.0),
        );
        assert!(d.is_none() || d.unwrap() < 0.0);
    }

    #[test]
    fn ray_line_upward() {
        // Ray from (5,0) going up (pi/2), segment from (0,5) to (10,5)
        let d = ray_line_intersect(
            Vec2::new(5.0, 0.0),
            FRAC_PI_2,
            Vec2::new(0.0, 5.0),
            Vec2::new(10.0, 5.0),
        );
        assert!(d.is_some());
        assert!((d.unwrap() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn ray_circle_hit() {
        let d = ray_circle_intersect(
            Vec2::new(0.0, 0.0),
            0.0,
            Vec2::new(5.0, 0.0),
            0.5,
        );
        assert!(d.is_some());
        assert!((d.unwrap() - 4.5).abs() < 1e-6);
    }

    #[test]
    fn ray_circle_miss() {
        let d = ray_circle_intersect(
            Vec2::new(0.0, 0.0),
            0.0,
            Vec2::new(5.0, 5.0),
            0.5,
        );
        assert!(d.is_none());
    }

    #[test]
    fn ray_circle_behind() {
        // Circle behind the ray origin
        let d = ray_circle_intersect(
            Vec2::new(0.0, 0.0),
            0.0,
            Vec2::new(-5.0, 0.0),
            0.5,
        );
        assert!(d.is_none());
    }

    #[test]
    fn ray_circle_upward() {
        let d = ray_circle_intersect(
            Vec2::new(0.0, 0.0),
            FRAC_PI_2,
            Vec2::new(0.0, 3.0),
            1.0,
        );
        assert!(d.is_some());
        assert!((d.unwrap() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn parallel_ray_line() {
        // Ray along x-axis, segment also along x-axis
        let d = ray_line_intersect(
            Vec2::new(0.0, 0.0),
            0.0,
            Vec2::new(3.0, 0.0),
            Vec2::new(5.0, 0.0),
        );
        assert!(d.is_none());
    }
}
