//! Carokia 2D Simulation
//!
//! A built-in pure-Rust 2D simulation environment for the Carokia brain.
//! Provides a virtual world with walls, objects, a simulated robot with lidar,
//! collision physics, and an ASCII terminal renderer.

pub mod physics;
pub mod renderer;
pub mod robot;
pub mod world;

pub use renderer::AsciiRenderer;
pub use robot::SimRobot;
pub use world::{ObjectKind, SimObject, Vec2, Wall, World};

/// Top-level simulation that ties world, robot, and rendering together.
pub struct Simulation {
    pub world: World,
    pub robot: SimRobot,
    pub tick_rate: f64,
    renderer: AsciiRenderer,
}

impl Simulation {
    /// Create a default simulation: a 10x10 room with some obstacles and a target.
    pub fn new_default() -> Self {
        let mut world = World::simple_room(10.0, 10.0);
        world.add_object(SimObject {
            id: "box1".into(),
            position: Vec2::new(3.0, 3.0),
            kind: ObjectKind::Obstacle,
            radius: 0.5,
        });
        world.add_object(SimObject {
            id: "box2".into(),
            position: Vec2::new(6.0, 4.0),
            kind: ObjectKind::Obstacle,
            radius: 0.5,
        });
        world.add_object(SimObject {
            id: "target".into(),
            position: Vec2::new(8.0, 8.0),
            kind: ObjectKind::Target,
            radius: 0.3,
        });

        let robot = SimRobot::new(1.5, 1.5);

        Self {
            world,
            robot,
            tick_rate: 10.0,
            renderer: AsciiRenderer::new(40, 20),
        }
    }

    /// Create a simulation with a custom world, robot, and rendering size.
    pub fn new(
        world: World,
        robot: SimRobot,
        tick_rate: f64,
        render_width: usize,
        render_height: usize,
    ) -> Self {
        Self {
            world,
            robot,
            tick_rate,
            renderer: AsciiRenderer::new(render_width, render_height),
        }
    }

    /// Advance the simulation by one tick with the given velocity commands.
    pub fn tick(&mut self, linear_vel: f64, angular_vel: f64) {
        let dt = 1.0 / self.tick_rate;
        self.robot.step(linear_vel, angular_vel, dt, &self.world);
    }

    /// Render the current state to an ASCII string.
    pub fn render(&self) -> String {
        self.renderer.render(&self.world, &self.robot)
    }

    /// Get a lidar scan from the robot's current position.
    pub fn lidar_scan(&self) -> Vec<f64> {
        self.robot.scan_lidar(&self.world)
    }

    /// Get objects within the given range of the robot.
    pub fn nearby_objects(&self, range: f64) -> Vec<&SimObject> {
        self.robot.detect_objects(&self.world, range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_new_default() {
        let sim = Simulation::new_default();
        assert!((sim.world.width - 10.0).abs() < 1e-10);
        assert!((sim.world.height - 10.0).abs() < 1e-10);
        assert_eq!(sim.world.objects.len(), 3);
    }

    #[test]
    fn simulation_tick_advances_position() {
        let mut sim = Simulation::new_default();
        let start = sim.robot.position;
        sim.tick(1.0, 0.0);
        // Robot should have moved east
        assert!(sim.robot.position.x > start.x);
    }

    #[test]
    fn simulation_render_non_empty() {
        let sim = Simulation::new_default();
        let output = sim.render();
        assert!(!output.is_empty());
        assert!(output.contains('@'));
    }

    #[test]
    fn simulation_lidar_scan() {
        let sim = Simulation::new_default();
        let scan = sim.lidar_scan();
        assert_eq!(scan.len(), sim.robot.lidar_rays);
    }

    #[test]
    fn simulation_nearby_objects() {
        let sim = Simulation::new_default();
        let near = sim.nearby_objects(3.0);
        // box1 at (3,3) is ~2.1 from robot at (1.5,1.5) — within range
        assert!(!near.is_empty());
    }

    #[test]
    fn simulation_multiple_ticks() {
        let mut sim = Simulation::new_default();
        for _ in 0..100 {
            sim.tick(1.0, 0.1);
        }
        // Robot should still be inside the room
        assert!(sim.robot.position.x >= 0.0);
        assert!(sim.robot.position.x <= sim.world.width);
        assert!(sim.robot.position.y >= 0.0);
        assert!(sim.robot.position.y <= sim.world.height);
    }
}
