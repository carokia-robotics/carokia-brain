//! Guardian mode demo — autonomous patrol, threat detection, and emergency response.
//!
//! Run with: cargo run --example guardian_demo -p carokia-brain --features simulation
//!
//! With clap CLI:
//!   cargo run --example guardian_demo -p carokia-brain --features "simulation,cli" -- --max-ticks 200
//!
//! The robot patrols waypoints around a room. After ~50 ticks an intruder
//! (unknown person) is spawned. The threat detection system escalates through
//! Suspicious -> Confirmed, triggering an emergency halt and alert.

#[cfg(feature = "simulation")]
#[tokio::main]
async fn main() {
    use carokia_brain::alerts::AlertManager;
    use carokia_decision::{Behavior, PatrolBehavior, ThreatDetectionBehavior, ThreatLevel};
    use carokia_sim::{ObjectKind, SimObject, Simulation, Vec2, World};

    #[cfg(feature = "cli")]
    use clap::Parser;

    #[cfg(feature = "cli")]
    #[derive(Parser)]
    #[command(name = "carokia-guardian", about = "Carokia guardian mode demo")]
    struct Args {
        /// Maximum simulation ticks
        #[arg(long, default_value_t = 200)]
        max_ticks: usize,
        /// Tick at which the intruder spawns
        #[arg(long, default_value_t = 50)]
        intruder_tick: usize,
        /// Sustained detection ticks before confirmation
        #[arg(long, default_value_t = 5)]
        sustained_ticks: usize,
    }

    #[cfg(feature = "cli")]
    let args = Args::parse();

    #[cfg(feature = "cli")]
    let (max_ticks, intruder_spawn_tick, sustained_ticks) =
        (args.max_ticks, args.intruder_tick, args.sustained_ticks);

    #[cfg(not(feature = "cli"))]
    let (max_ticks, intruder_spawn_tick, sustained_ticks) = (200_usize, 50_usize, 5_usize);

    // --- Build the world ---
    let mut world = World::simple_room(12.0, 10.0);

    // Add furniture / obstacles
    world.add_object(SimObject {
        id: "desk".into(),
        position: Vec2::new(3.0, 3.0),
        kind: ObjectKind::Obstacle,
        radius: 0.6,
    });
    world.add_object(SimObject {
        id: "cabinet".into(),
        position: Vec2::new(9.0, 2.0),
        kind: ObjectKind::Obstacle,
        radius: 0.5,
    });
    world.add_object(SimObject {
        id: "pillar".into(),
        position: Vec2::new(6.0, 7.0),
        kind: ObjectKind::Obstacle,
        radius: 0.4,
    });

    // --- Set up patrol waypoints (clockwise around the room) ---
    let waypoints = vec![(2.0, 2.0), (10.0, 2.0), (10.0, 8.0), (2.0, 8.0)];

    let robot = carokia_sim::SimRobot::new(2.0, 2.0);
    let mut sim = Simulation::new(world, robot, 10.0, 48, 20);

    // --- Behaviors ---
    let patrol = PatrolBehavior::new(waypoints.clone());
    let threat_detector = ThreatDetectionBehavior::new(6.0, sustained_ticks);
    let mut alert_manager = AlertManager::new();

    // Track state
    let mut mode = "PATROL";
    let mut intruder_spawned = false;
    let mut intruder_pos: Option<(f64, f64)> = None;

    for tick in 0..max_ticks {
        // --- Spawn intruder after N ticks ---
        if tick == intruder_spawn_tick && !intruder_spawned {
            let pos = Vec2::new(7.0, 4.0);
            sim.world.add_object(SimObject {
                id: "intruder".into(),
                position: pos,
                kind: ObjectKind::Person {
                    name: None,
                    is_known: false,
                },
                radius: 0.3,
            });
            intruder_spawned = true;
            intruder_pos = Some((pos.x, pos.y));
        }

        // --- Threat detection: check for unknown persons in range ---
        let robot_pos = sim.robot.position;

        // Detect unknown persons within alert distance
        let nearby = sim.nearby_objects(6.0);
        let unknown_person_nearby = nearby.iter().any(|obj| {
            matches!(
                obj.kind,
                ObjectKind::Person {
                    is_known: false,
                    ..
                }
            )
        });

        // Update threat detector by simulating its evaluate
        if unknown_person_nearby {
            // Feed the detector by calling evaluate with a percept
            use carokia_core::Modality;
            use carokia_decision::WorldState;
            use carokia_perception::{Percept, PerceptContent};

            let person_obj = nearby.iter().find(|obj| {
                matches!(
                    obj.kind,
                    ObjectKind::Person {
                        is_known: false,
                        ..
                    }
                )
            });

            if let Some(person) = person_obj {
                let dist = robot_pos.distance_to(person.position);
                let mut ws = WorldState::new();
                ws.percepts.push(Percept::new(
                    Modality::Vision,
                    PerceptContent::Person {
                        name: None,
                        distance: dist,
                        bearing: 0.0,
                    },
                    0.95,
                ));
                let _ = threat_detector.evaluate(&ws).await;
            }
        } else {
            // No person nearby — let detector decay
            let ws = carokia_decision::WorldState::new();
            let _ = threat_detector.evaluate(&ws).await;
        }

        let threat_level = threat_detector.threat_level();

        // --- Update mode and alerts ---
        match threat_level {
            ThreatLevel::Confirmed => {
                if mode != "EMERGENCY" {
                    mode = "EMERGENCY";
                    let loc = intruder_pos;
                    alert_manager.raise(
                        ThreatLevel::Confirmed,
                        "Intruder detected — threat confirmed! Robot halted.",
                        loc,
                    );
                }
            }
            ThreatLevel::Suspicious => {
                if mode != "ALERT" && mode != "EMERGENCY" {
                    mode = "ALERT";
                    alert_manager.raise(
                        ThreatLevel::Suspicious,
                        "Unknown person detected — monitoring...",
                        intruder_pos,
                    );
                }
            }
            ThreatLevel::None => {
                if mode != "PATROL" && mode != "EMERGENCY" {
                    mode = "PATROL";
                }
            }
        }

        // --- Robot movement ---
        let (linear, angular) = if mode == "EMERGENCY" {
            // Halted
            (0.0, 0.0)
        } else {
            // Navigate toward current patrol waypoint
            let (tx, ty) = patrol.current_target();
            let dx = tx - robot_pos.x;
            let dy = ty - robot_pos.y;
            // Check arrival
            if patrol.has_arrived(robot_pos.x, robot_pos.y) {
                patrol.advance();
            }

            let target_angle = dy.atan2(dx);
            let mut angle_diff = target_angle - sim.robot.heading;
            while angle_diff > std::f64::consts::PI {
                angle_diff -= std::f64::consts::TAU;
            }
            while angle_diff < -std::f64::consts::PI {
                angle_diff += std::f64::consts::TAU;
            }

            // Lidar-based obstacle avoidance
            let lidar = sim.lidar_scan();
            let front_min = lidar[0].min(lidar[1]).min(lidar[lidar.len() - 1]);

            if front_min < 1.0 {
                let left_avg: f64 = lidar[1..=3].iter().sum::<f64>() / 3.0;
                let right_avg: f64 = lidar[lidar.len() - 3..].iter().sum::<f64>() / 3.0;
                let turn_dir = if left_avg > right_avg { 1.0 } else { -1.0 };
                (0.3, turn_dir * 2.0)
            } else if front_min < 2.0 {
                (0.8, angle_diff.clamp(-1.5, 1.5))
            } else {
                (1.8, angle_diff.clamp(-2.0, 2.0))
            }
        };

        sim.tick(linear, angular);

        // --- Render ---
        print!("\x1B[2J\x1B[H");

        let world_render = sim.render();
        let pos = sim.robot.position;
        let heading_deg = sim.robot.heading.to_degrees();
        let wp_idx = patrol.current_index() + 1;
        let wp_total = patrol.waypoint_count();

        // Dashboard
        let width = 50;
        let h_line: String = "=".repeat(width);
        let threat_display = match threat_level {
            ThreatLevel::None => "NONE".to_string(),
            ThreatLevel::Suspicious => "!! SUSPICIOUS".to_string(),
            ThreatLevel::Confirmed => "!!! CONFIRMED".to_string(),
        };

        println!("+{}+", h_line);
        println!("|{:^width$}|", "CAROKIA GUARDIAN MODE", width = width);
        println!("+{}+", h_line);

        // Print world render with borders
        for line in world_render.lines() {
            println!("| {:<w$}|", line, w = width - 1);
        }

        println!("+{}+", h_line);
        println!(
            "| Position: ({:.1}, {:.1})  Heading: {:.0}deg{:>pad$}|",
            pos.x,
            pos.y,
            heading_deg,
            "",
            pad = width.saturating_sub(42)
        );
        println!(
            "| Mode: {:<10} Waypoint: {}/{}               |",
            mode, wp_idx, wp_total
        );
        println!(
            "| Tick: {:>4}/{:<4}  Speed: {:.1}{:>pad$}|",
            tick,
            max_ticks,
            sim.robot.speed,
            "",
            pad = width.saturating_sub(32)
        );
        println!("| Threat: {:<w$}|", threat_display, w = width - 10);
        println!("| Alerts: {:<w$}|", alert_manager.count(), w = width - 10);

        // Show recent alerts
        let recent = alert_manager.recent(3);
        if !recent.is_empty() {
            println!("+{}+", "-".repeat(width));
            for alert in recent {
                let loc_str = match alert.location {
                    Some((x, y)) => format!(" at ({:.1}, {:.1})", x, y),
                    None => String::new(),
                };
                let msg = format!("[{}] {}{}", alert.level, alert.message, loc_str);
                // Truncate if too long
                let display_msg = if msg.len() > width - 3 {
                    format!("{}...", &msg[..width - 6])
                } else {
                    msg
                };
                println!("| {:<w$}|", display_msg, w = width - 1);
            }
        }

        println!("+{}+", h_line);

        // Status line
        if mode == "EMERGENCY" {
            println!();
            println!("  >>> ROBOT HALTED — INTRUDER DETECTED <<<");
            println!("  >>> Monitoring continues...            <<<");
        } else if intruder_spawned && mode == "ALERT" {
            println!();
            println!("  >>> Unknown person detected — assessing threat... <<<");
        } else if !intruder_spawned {
            let ticks_until = intruder_spawn_tick.saturating_sub(tick);
            if ticks_until > 0 && ticks_until <= 10 {
                println!();
                println!("  [Intruder spawns in {} ticks...]", ticks_until);
            }
        }

        // Exit early after emergency is shown for a while
        if mode == "EMERGENCY" && tick > intruder_spawn_tick + sustained_ticks + 20 {
            println!();
            println!("  Demo complete. All systems nominal.");
            println!("  Guardian detected and responded to the intruder autonomously.");
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!();
    println!("=== Guardian Demo Summary ===");
    println!("Total alerts raised: {}", alert_manager.count());
    for alert in alert_manager.all() {
        let loc = match alert.location {
            Some((x, y)) => format!("({:.1}, {:.1})", x, y),
            None => "N/A".into(),
        };
        println!(
            "  [{:>10}] {} (location: {})",
            format!("{}", alert.level),
            alert.message,
            loc,
        );
    }
    println!();
    println!("Carokia Guardian Mode — Sprint 6 complete.");
}

#[cfg(not(feature = "simulation"))]
fn main() {
    eprintln!("This example requires the 'simulation' feature.");
    eprintln!("Run with: cargo run --example guardian_demo -p carokia-brain --features simulation");
}
