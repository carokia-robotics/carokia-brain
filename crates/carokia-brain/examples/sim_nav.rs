//! Autonomous navigation demo using the built-in 2D simulation.
//!
//! Run with: cargo run --example sim_nav -p carokia-brain --features simulation
//!
//! With clap CLI:
//!   cargo run --example sim_nav -p carokia-brain --features "simulation,cli" -- --max-ticks 300
//!
//! The robot navigates from its starting position toward a target,
//! avoiding obstacles using lidar-based steering.

#[cfg(feature = "simulation")]
#[tokio::main]
async fn main() {
    use carokia_sim::{Simulation, Vec2};

    #[cfg(feature = "cli")]
    use clap::Parser;

    #[cfg(feature = "cli")]
    #[derive(Parser)]
    #[command(name = "carokia-sim-nav", about = "Carokia autonomous navigation demo")]
    struct Args {
        /// Maximum simulation ticks
        #[arg(long, default_value_t = 500)]
        max_ticks: usize,
        /// Target X coordinate
        #[arg(long, default_value_t = 8.0)]
        target_x: f64,
        /// Target Y coordinate
        #[arg(long, default_value_t = 8.0)]
        target_y: f64,
    }

    #[cfg(feature = "cli")]
    let args = Args::parse();

    #[cfg(feature = "cli")]
    let (max_ticks, target) = (args.max_ticks, Vec2::new(args.target_x, args.target_y));

    #[cfg(not(feature = "cli"))]
    let (max_ticks, target) = (500, Vec2::new(8.0, 8.0));

    let mut sim = Simulation::new_default();

    println!("Carokia 2D Simulation - Autonomous Navigation Demo");
    println!("Robot '@' navigating to target 'T' while avoiding obstacles 'O'");
    println!("---");

    for tick in 0..max_ticks {
        // Clear screen and move cursor to top-left.
        print!("\x1B[2J\x1B[H");

        // Render the world.
        println!("{}", sim.render());

        // Status display.
        let pos = sim.robot.position;
        let dx = target.x - pos.x;
        let dy = target.y - pos.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let heading_deg = sim.robot.heading.to_degrees();

        println!(
            "Tick: {:>4}  Position: ({:.1}, {:.1})  Heading: {:.0}deg  Distance to target: {:.2}",
            tick, pos.x, pos.y, heading_deg, dist
        );

        // Check if target reached.
        if dist < 0.8 {
            println!("TARGET REACHED! Navigation complete.");
            break;
        }

        // Lidar-based navigation.
        let lidar = sim.lidar_scan();
        let target_angle = dy.atan2(dx);

        // Angle difference (normalized to [-pi, pi]).
        let mut angle_diff = target_angle - sim.robot.heading;
        while angle_diff > std::f64::consts::PI {
            angle_diff -= std::f64::consts::TAU;
        }
        while angle_diff < -std::f64::consts::PI {
            angle_diff += std::f64::consts::TAU;
        }

        // Check front lidar rays for obstacles.
        // Rays 0, 1, and last are roughly forward.
        let front_min = lidar[0].min(lidar[1]).min(lidar[lidar.len() - 1]);

        let (linear, angular) = if front_min < 1.0 {
            // Obstacle close ahead — stop and turn away.
            // Determine which side is more open.
            let left_avg: f64 = lidar[1..=3].iter().sum::<f64>() / 3.0;
            let right_avg: f64 = lidar[lidar.len() - 3..].iter().sum::<f64>() / 3.0;
            let turn_dir = if left_avg > right_avg { 1.0 } else { -1.0 };
            (0.2, turn_dir * 2.0)
        } else if front_min < 2.0 {
            // Obstacle approaching — slow down and steer toward target.
            (0.5, angle_diff.clamp(-1.5, 1.5))
        } else {
            // Clear path — move toward target.
            (1.5, angle_diff.clamp(-2.0, 2.0))
        };

        // Print lidar summary.
        let nearby = sim.nearby_objects(3.0);
        if !nearby.is_empty() {
            let names: Vec<&str> = nearby.iter().map(|o| o.id.as_str()).collect();
            println!("Nearby objects: {}", names.join(", "));
        }
        println!(
            "Front clearance: {:.2}  Command: linear={:.1} angular={:.1}",
            front_min, linear, angular
        );

        sim.tick(linear, angular);

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

#[cfg(not(feature = "simulation"))]
fn main() {
    eprintln!("This example requires the 'simulation' feature.");
    eprintln!("Run with: cargo run --example sim_nav -p carokia-brain --features simulation");
}
