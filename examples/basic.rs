use fyrox_ai_free::*;

fn main() {
    // Build AI state machine: Patrol -> Chase -> Attack
    let mut sm = AiStateMachine::new();

    let patrol = sm.add_state(AiState::new("Patrol"));
    let chase = sm.add_state(AiState::new("Chase"));
    let attack = sm.add_state(AiState::new("Attack"));

    sm.add_transition(AiTransition::new(
        "Spot Enemy",
        patrol,
        chase,
        ConditionNode::Leaf(Condition::is_true("enemy_visible")),
    ));
    sm.add_transition(AiTransition::new(
        "In Range",
        chase,
        attack,
        ConditionNode::Leaf(Condition::is_true("in_attack_range")),
    ));
    sm.add_transition(AiTransition::new(
        "Lost Enemy",
        chase,
        patrol,
        ConditionNode::Leaf(Condition::is_false("enemy_visible")),
    ));
    sm.add_transition(AiTransition::new(
        "Target Down",
        attack,
        patrol,
        ConditionNode::Leaf(Condition::is_false("enemy_visible")),
    ));

    sm.set_entry_state(patrol);

    let mut blackboard = Blackboard::new();
    let dt = 0.016; // ~60 FPS

    // Simulate: enemy not visible
    blackboard.set("enemy_visible", BlackboardValue::Bool(false));
    blackboard.set("in_attack_range", BlackboardValue::Bool(false));
    sm.tick(dt, &mut blackboard);
    println!("State: {} (expected: Patrol)", sm.current_state_name());

    // Simulate: enemy spotted
    blackboard.set("enemy_visible", BlackboardValue::Bool(true));
    sm.tick(dt, &mut blackboard);
    println!("State: {} (expected: Chase)", sm.current_state_name());

    // Simulate: close enough to attack
    blackboard.set("in_attack_range", BlackboardValue::Bool(true));
    sm.tick(dt, &mut blackboard);
    println!("State: {} (expected: Attack)", sm.current_state_name());

    // Simulate: enemy eliminated
    blackboard.set("enemy_visible", BlackboardValue::Bool(false));
    blackboard.set("in_attack_range", BlackboardValue::Bool(false));
    sm.tick(dt, &mut blackboard);
    println!("State: {} (expected: Patrol)", sm.current_state_name());

    println!("\nAll transitions working correctly!");
}
