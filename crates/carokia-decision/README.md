# carokia-decision

Behavior engine and decision-making for Carokia. Evaluates the current world state (percepts, goals, tasks, memories) and selects the highest-priority action. Includes patrol behavior for waypoint navigation, threat detection with escalation levels (None, Suspicious, Confirmed), and emergency response handling. The `BehaviorDecisionEngine` coordinates multiple behaviors with priority-based arbitration.
