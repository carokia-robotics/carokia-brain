# Demo Script: Guardian Mode

## Overview
Show Carokia's full autonomous guardian capability: waypoint patrol, threat detection with escalation, and emergency response. Runs in a 2D simulation with a live ASCII terminal dashboard.

## Prerequisites
- No external dependencies -- the simulation runs entirely in-process.

## Commands

```bash
cargo run --example guardian_demo -p carokia-brain --features simulation
```

## Walkthrough

1. **Launch** -- the terminal shows a 2D ASCII grid with the robot (R), walls, furniture, and waypoints.

2. **Patrol phase (ticks 1-49)** -- watch the robot move between waypoints in a clockwise pattern.
   - Expected: The robot navigates around obstacles, the dashboard shows `Mode: Patrol` and current waypoint index.

3. **Intruder spawns (tick ~50)** -- an unknown person (P) appears in the room.
   - Expected: The threat detection system picks up the intruder. Status escalates from `Clear` to `Suspicious`.

4. **Threat confirmed** -- after sustained detection (5 ticks), threat level reaches `Confirmed`.
   - Expected: The robot halts patrol, the alert manager fires an emergency alert, dashboard shows `EMERGENCY`.

5. **Simulation ends** -- the demo runs for 200 ticks and exits with a summary.

## Talking Points

- The behavior engine is composable: `PatrolBehavior` and `ThreatDetectionBehavior` implement the same `Behavior` trait and can be layered.
- Threat escalation requires sustained detection to avoid false alarms -- a brief sensor ghost will not trigger emergency.
- The alert manager is designed to integrate with real notification systems (SMS, sirens, radio) via trait implementations.
- The entire simulation runs without an LLM -- pure Rust logic for deterministic, testable behavior.
