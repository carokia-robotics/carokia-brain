# Demo Script: Navigation Simulation

## Overview
Show Carokia navigating autonomously in a 2D simulated world with lidar-based obstacle avoidance. The world, robot, and physics are rendered live in the terminal as ASCII art.

## Prerequisites
- No external dependencies -- everything runs in-process.

## Commands

```bash
cargo run --example sim_nav -p carokia-brain --features simulation
```

## Walkthrough

1. **Launch** -- a 2D room appears in the terminal with walls, obstacles, and the robot (R).

2. **Autonomous movement** -- the robot begins navigating toward a goal point.
   - Expected: The robot moves through open space, and lidar rays are cast to detect nearby obstacles.

3. **Obstacle avoidance** -- watch as the robot approaches furniture or walls.
   - Expected: Lidar detects the obstacle at range; the robot steers around it rather than colliding.

4. **Goal reached** -- the robot arrives at the target position.
   - Expected: The demo prints a success message or the robot selects a new goal.

5. **Simulation ends** -- runs for a fixed number of ticks and exits.

## Talking Points

- The simulation crate (`carokia-sim`) models 2D physics: position, velocity, heading, and collision.
- Lidar is simulated with ray-casting against world geometry -- the same interface a real lidar sensor would expose.
- The ASCII renderer updates in-place using terminal escape codes for a smooth live display.
- This simulation is the testbed for all navigation algorithms before they run on real hardware.
