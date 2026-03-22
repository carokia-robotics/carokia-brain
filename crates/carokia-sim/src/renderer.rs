use crate::robot::SimRobot;
use crate::world::{ObjectKind, World};

/// ASCII renderer that displays the simulation world in the terminal.
pub struct AsciiRenderer {
    width: usize,
    height: usize,
}

impl AsciiRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Render the world and robot to an ASCII string.
    ///
    /// Symbols:
    /// - `#` wall
    /// - `O` obstacle
    /// - `T` target
    /// - `P` person
    /// - `@` robot (with directional indicator)
    /// - `.` lidar hit point
    /// - ` ` empty space
    pub fn render(&self, world: &World, robot: &SimRobot) -> String {
        let mut grid = vec![vec![' '; self.width]; self.height];

        let scale_x = self.width as f64 / world.width;
        let scale_y = self.height as f64 / world.height;

        // Draw walls by sampling points along each wall segment.
        for wall in &world.walls {
            let steps = ((wall.start.distance_to(wall.end))
                * scale_x.max(scale_y))
                .ceil() as usize;
            let steps = steps.max(1);
            for i in 0..=steps {
                let t = i as f64 / steps as f64;
                let wx = wall.start.x + t * (wall.end.x - wall.start.x);
                let wy = wall.start.y + t * (wall.end.y - wall.start.y);
                let (gx, gy) = self.world_to_grid(wx, wy, scale_x, scale_y);
                if gx < self.width && gy < self.height {
                    grid[gy][gx] = '#';
                }
            }
        }

        // Draw objects.
        for obj in &world.objects {
            let (gx, gy) = self.world_to_grid(obj.position.x, obj.position.y, scale_x, scale_y);
            if gx < self.width && gy < self.height {
                let ch = match obj.kind {
                    ObjectKind::Obstacle => 'O',
                    ObjectKind::Target => 'T',
                    ObjectKind::Person { .. } => 'P',
                };
                grid[gy][gx] = ch;
            }
        }

        // Draw lidar hits.
        let lidar = robot.scan_lidar(world);
        let angle_step = std::f64::consts::TAU / robot.lidar_rays as f64;
        for (i, &dist) in lidar.iter().enumerate() {
            if dist < robot.lidar_range {
                let angle = robot.heading + i as f64 * angle_step;
                let hx = robot.position.x + dist * angle.cos();
                let hy = robot.position.y + dist * angle.sin();
                let (gx, gy) = self.world_to_grid(hx, hy, scale_x, scale_y);
                if gx < self.width && gy < self.height && grid[gy][gx] == ' ' {
                    grid[gy][gx] = '.';
                }
            }
        }

        // Draw robot.
        let (rx, ry) = self.world_to_grid(robot.position.x, robot.position.y, scale_x, scale_y);
        if rx < self.width && ry < self.height {
            grid[ry][rx] = '@';
        }

        // Draw direction indicator one cell ahead of robot.
        let dir_char = self.heading_char(robot.heading);
        let ahead_x = robot.position.x + 0.5 / scale_x.max(0.01) * robot.heading.cos();
        let ahead_y = robot.position.y + 0.5 / scale_y.max(0.01) * robot.heading.sin();
        let (dx, dy) = self.world_to_grid(ahead_x, ahead_y, scale_x, scale_y);
        if dx < self.width && dy < self.height && grid[dy][dx] == ' ' {
            grid[dy][dx] = dir_char;
        }

        // Render grid top-to-bottom (row 0 = top of screen = max y in world).
        grid.iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Convert world coordinates to grid coordinates.
    /// Grid y is inverted: row 0 = top = max world y.
    fn world_to_grid(
        &self,
        wx: f64,
        wy: f64,
        scale_x: f64,
        scale_y: f64,
    ) -> (usize, usize) {
        let gx = (wx * scale_x).round() as isize;
        let gy = (self.height as f64 - 1.0 - wy * scale_y).round() as isize;
        let gx = gx.clamp(0, self.width as isize - 1) as usize;
        let gy = gy.clamp(0, self.height as isize - 1) as usize;
        (gx, gy)
    }

    fn heading_char(&self, heading: f64) -> char {
        // Normalize to [0, 2pi).
        let h = heading.rem_euclid(std::f64::consts::TAU);
        let eighth = std::f64::consts::TAU / 8.0;
        if h < eighth || h >= 7.0 * eighth {
            '>'
        } else if h < 3.0 * eighth {
            '^'
        } else if h < 5.0 * eighth {
            '<'
        } else {
            'v'
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{ObjectKind, SimObject, Vec2, World};

    #[test]
    fn render_non_empty() {
        let world = World::simple_room(10.0, 10.0);
        let robot = SimRobot::new(5.0, 5.0);
        let renderer = AsciiRenderer::new(40, 20);
        let output = renderer.render(&world, &robot);
        assert!(!output.is_empty());
        assert!(output.contains('@')); // Robot
        assert!(output.contains('#')); // Walls
    }

    #[test]
    fn render_shows_objects() {
        let mut world = World::simple_room(10.0, 10.0);
        world.add_object(SimObject {
            id: "obs".into(),
            position: Vec2::new(3.0, 3.0),
            kind: ObjectKind::Obstacle,
            radius: 0.5,
        });
        world.add_object(SimObject {
            id: "tgt".into(),
            position: Vec2::new(7.0, 7.0),
            kind: ObjectKind::Target,
            radius: 0.3,
        });
        let robot = SimRobot::new(5.0, 5.0);
        let renderer = AsciiRenderer::new(40, 20);
        let output = renderer.render(&world, &robot);
        assert!(output.contains('O'));
        assert!(output.contains('T'));
    }

    #[test]
    fn render_correct_dimensions() {
        let world = World::simple_room(10.0, 10.0);
        let robot = SimRobot::new(5.0, 5.0);
        let renderer = AsciiRenderer::new(30, 15);
        let output = renderer.render(&world, &robot);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 15);
        for line in &lines {
            assert_eq!(line.len(), 30);
        }
    }
}
