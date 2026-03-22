use serde::{Deserialize, Serialize};

use crate::physics;

/// A 2D vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(self, other: Vec2) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len < 1e-12 {
            Self::new(0.0, 0.0)
        } else {
            Self::new(self.x / len, self.y / len)
        }
    }

    pub fn dot(self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

/// A wall segment defined by two endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wall {
    pub start: Vec2,
    pub end: Vec2,
}

/// The kind of object in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectKind {
    Obstacle,
    Target,
    Person {
        name: Option<String>,
        is_known: bool,
    },
}

/// An object in the simulated world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimObject {
    pub id: String,
    pub position: Vec2,
    pub kind: ObjectKind,
    pub radius: f64,
}

/// The 2D simulation world containing walls and objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub width: f64,
    pub height: f64,
    pub walls: Vec<Wall>,
    pub objects: Vec<SimObject>,
}

impl World {
    /// Create a simple rectangular room bounded by four walls.
    pub fn simple_room(width: f64, height: f64) -> Self {
        let walls = vec![
            // Bottom
            Wall {
                start: Vec2::new(0.0, 0.0),
                end: Vec2::new(width, 0.0),
            },
            // Right
            Wall {
                start: Vec2::new(width, 0.0),
                end: Vec2::new(width, height),
            },
            // Top
            Wall {
                start: Vec2::new(width, height),
                end: Vec2::new(0.0, height),
            },
            // Left
            Wall {
                start: Vec2::new(0.0, height),
                end: Vec2::new(0.0, 0.0),
            },
        ];
        Self {
            width,
            height,
            walls,
            objects: Vec::new(),
        }
    }

    /// Add an object to the world.
    pub fn add_object(&mut self, obj: SimObject) {
        self.objects.push(obj);
    }

    /// Check if a circle at `pos` with given `radius` collides with any wall or object.
    pub fn check_collision(&self, pos: Vec2, radius: f64) -> bool {
        // Check wall collisions.
        for wall in &self.walls {
            if physics::line_circle_intersect(wall.start, wall.end, pos, radius) {
                return true;
            }
        }
        // Check object collisions.
        for obj in &self.objects {
            let dist = pos.distance_to(obj.position);
            if dist < radius + obj.radius {
                return true;
            }
        }
        false
    }

    /// Cast a ray from `origin` at `angle` (radians) and return the distance to the
    /// nearest wall or object, capped at `max_range`.
    pub fn raycast(&self, origin: Vec2, angle: f64, max_range: f64) -> f64 {
        let mut min_dist = max_range;

        // Check walls.
        for wall in &self.walls {
            if let Some(d) = physics::ray_line_intersect(origin, angle, wall.start, wall.end) {
                if d > 0.0 && d < min_dist {
                    min_dist = d;
                }
            }
        }

        // Check objects (treat as circles: find ray-circle intersection).
        for obj in &self.objects {
            if let Some(d) = physics::ray_circle_intersect(origin, angle, obj.position, obj.radius)
            {
                if d > 0.0 && d < min_dist {
                    min_dist = d;
                }
            }
        }

        min_dist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec2_distance() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);
        assert!((a.distance_to(b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn vec2_normalize() {
        let v = Vec2::new(3.0, 4.0);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn vec2_normalize_zero() {
        let v = Vec2::new(0.0, 0.0);
        let n = v.normalize();
        assert!((n.length()).abs() < 1e-10);
    }

    #[test]
    fn simple_room_has_four_walls() {
        let world = World::simple_room(10.0, 10.0);
        assert_eq!(world.walls.len(), 4);
    }

    #[test]
    fn simple_room_dimensions() {
        let world = World::simple_room(10.0, 8.0);
        assert!((world.width - 10.0).abs() < 1e-10);
        assert!((world.height - 8.0).abs() < 1e-10);
    }

    #[test]
    fn collision_with_wall() {
        let world = World::simple_room(10.0, 10.0);
        // Near the left wall
        assert!(world.check_collision(Vec2::new(0.1, 5.0), 0.3));
        // Center of room — no collision
        assert!(!world.check_collision(Vec2::new(5.0, 5.0), 0.3));
    }

    #[test]
    fn collision_with_object() {
        let mut world = World::simple_room(10.0, 10.0);
        world.add_object(SimObject {
            id: "box".into(),
            position: Vec2::new(5.0, 5.0),
            kind: ObjectKind::Obstacle,
            radius: 0.5,
        });
        // Overlapping with object
        assert!(world.check_collision(Vec2::new(5.3, 5.0), 0.3));
        // Far from object
        assert!(!world.check_collision(Vec2::new(2.0, 2.0), 0.3));
    }

    #[test]
    fn raycast_hits_wall() {
        let world = World::simple_room(10.0, 10.0);
        // From center, cast right — should hit right wall at x=10
        let dist = world.raycast(Vec2::new(5.0, 5.0), 0.0, 20.0);
        assert!((dist - 5.0).abs() < 1e-6);
    }

    #[test]
    fn raycast_hits_top_wall() {
        let world = World::simple_room(10.0, 10.0);
        // From center, cast up (pi/2)
        let dist = world.raycast(Vec2::new(5.0, 5.0), std::f64::consts::FRAC_PI_2, 20.0);
        assert!((dist - 5.0).abs() < 1e-6);
    }

    #[test]
    fn raycast_max_range() {
        let world = World::simple_room(100.0, 100.0);
        // From center, cast right — wall at 50 but max_range is 10
        let dist = world.raycast(Vec2::new(50.0, 50.0), 0.0, 10.0);
        assert!((dist - 10.0).abs() < 1e-6);
    }

    #[test]
    fn raycast_hits_object() {
        let mut world = World::simple_room(20.0, 20.0);
        world.add_object(SimObject {
            id: "box".into(),
            position: Vec2::new(8.0, 5.0),
            kind: ObjectKind::Obstacle,
            radius: 0.5,
        });
        // From (3,5) cast right — should hit object surface at ~7.5
        let dist = world.raycast(Vec2::new(3.0, 5.0), 0.0, 20.0);
        assert!((dist - 4.5).abs() < 0.1);
    }
}
