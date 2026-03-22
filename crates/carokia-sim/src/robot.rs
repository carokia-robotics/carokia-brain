use crate::world::{SimObject, Vec2, World};

/// A simulated robot with position, heading, and sensors.
#[derive(Debug, Clone)]
pub struct SimRobot {
    pub position: Vec2,
    /// Heading in radians (0 = east, pi/2 = north).
    pub heading: f64,
    /// Collision radius of the robot body.
    pub radius: f64,
    /// Current speed (for display purposes).
    pub speed: f64,
    /// Number of lidar beams.
    pub lidar_rays: usize,
    /// Maximum lidar range in world units.
    pub lidar_range: f64,
}

impl SimRobot {
    /// Create a new robot at the given position facing east.
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            position: Vec2::new(x, y),
            heading: 0.0,
            radius: 0.3,
            speed: 0.0,
            lidar_rays: 12,
            lidar_range: 5.0,
        }
    }

    /// Advance the robot by `dt` seconds with the given linear and angular velocities.
    ///
    /// Uses sub-stepping to prevent tunneling through walls.
    /// The robot stops at the last valid position before a collision.
    /// Returns the resulting position.
    pub fn step(&mut self, linear_vel: f64, angular_vel: f64, dt: f64, world: &World) -> Vec2 {
        self.heading += angular_vel * dt;
        // Normalize heading to [0, 2*pi).
        self.heading = self.heading.rem_euclid(std::f64::consts::TAU);

        // Sub-step to prevent tunneling: break the movement into small increments.
        let total_dist = (linear_vel * dt).abs();
        let max_step = self.radius * 0.5; // Each sub-step moves at most half the robot radius.
        let num_steps = ((total_dist / max_step).ceil() as usize).max(1);
        let sub_dt = dt / num_steps as f64;

        let mut moved = false;
        for _ in 0..num_steps {
            let new_x = self.position.x + linear_vel * self.heading.cos() * sub_dt;
            let new_y = self.position.y + linear_vel * self.heading.sin() * sub_dt;
            let new_pos = Vec2::new(new_x, new_y);

            if !world.check_collision(new_pos, self.radius) {
                self.position = new_pos;
                moved = true;
            } else {
                // Stop at the first collision.
                break;
            }
        }

        self.speed = if moved { linear_vel } else { 0.0 };
        self.position
    }

    /// Simulate a lidar scan, returning distances for each ray evenly spaced around 360 degrees.
    ///
    /// Ray 0 is in the direction of the robot's heading.
    pub fn scan_lidar(&self, world: &World) -> Vec<f64> {
        let angle_step = std::f64::consts::TAU / self.lidar_rays as f64;
        (0..self.lidar_rays)
            .map(|i| {
                let angle = self.heading + i as f64 * angle_step;
                world.raycast(self.position, angle, self.lidar_range)
            })
            .collect()
    }

    /// Detect objects within the given range of the robot.
    pub fn detect_objects<'a>(&self, world: &'a World, range: f64) -> Vec<&'a SimObject> {
        world
            .objects
            .iter()
            .filter(|obj| self.position.distance_to(obj.position) <= range)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{ObjectKind, SimObject};

    #[test]
    fn robot_new_defaults() {
        let robot = SimRobot::new(1.0, 2.0);
        assert!((robot.position.x - 1.0).abs() < 1e-10);
        assert!((robot.position.y - 2.0).abs() < 1e-10);
        assert!((robot.heading).abs() < 1e-10);
        assert_eq!(robot.lidar_rays, 12);
    }

    #[test]
    fn robot_step_moves_forward() {
        let world = World::simple_room(10.0, 10.0);
        let mut robot = SimRobot::new(5.0, 5.0);
        // Heading = 0 (east), move forward 1 unit/s for 1 second
        robot.step(1.0, 0.0, 1.0, &world);
        assert!((robot.position.x - 6.0).abs() < 1e-6);
        assert!((robot.position.y - 5.0).abs() < 1e-6);
    }

    #[test]
    fn robot_step_blocked_by_wall() {
        let world = World::simple_room(10.0, 10.0);
        let mut robot = SimRobot::new(9.5, 5.0);
        // Try to move east into the right wall
        let pos = robot.step(5.0, 0.0, 1.0, &world);
        // Should not have moved through the wall.
        // The robot may advance slightly before the collision is detected,
        // but must remain inside the room with clearance for its radius.
        assert!(pos.x < 10.0 - robot.radius + 0.01);
        // Should have advanced at least a little from start
        assert!(pos.x >= 9.5);
    }

    #[test]
    fn robot_step_with_rotation() {
        let world = World::simple_room(10.0, 10.0);
        let mut robot = SimRobot::new(5.0, 5.0);
        // Rotate 90 degrees (pi/2), then move forward
        robot.step(0.0, std::f64::consts::FRAC_PI_2, 1.0, &world);
        assert!((robot.heading - std::f64::consts::FRAC_PI_2).abs() < 1e-6);
        // Now move forward (should go north / +y)
        robot.step(1.0, 0.0, 1.0, &world);
        assert!((robot.position.x - 5.0).abs() < 1e-6);
        assert!((robot.position.y - 6.0).abs() < 1e-6);
    }

    #[test]
    fn lidar_scan_correct_count() {
        let world = World::simple_room(10.0, 10.0);
        let robot = SimRobot::new(5.0, 5.0);
        let scan = robot.scan_lidar(&world);
        assert_eq!(scan.len(), robot.lidar_rays);
    }

    #[test]
    fn lidar_scan_values_within_range() {
        let world = World::simple_room(10.0, 10.0);
        let robot = SimRobot::new(5.0, 5.0);
        let scan = robot.scan_lidar(&world);
        for d in &scan {
            assert!(*d > 0.0);
            assert!(*d <= robot.lidar_range);
        }
    }

    #[test]
    fn detect_objects_in_range() {
        let mut world = World::simple_room(20.0, 20.0);
        world.add_object(SimObject {
            id: "near".into(),
            position: Vec2::new(3.0, 5.0),
            kind: ObjectKind::Obstacle,
            radius: 0.5,
        });
        world.add_object(SimObject {
            id: "far".into(),
            position: Vec2::new(15.0, 15.0),
            kind: ObjectKind::Target,
            radius: 0.3,
        });
        let robot = SimRobot::new(5.0, 5.0);
        let detected = robot.detect_objects(&world, 5.0);
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].id, "near");
    }
}
