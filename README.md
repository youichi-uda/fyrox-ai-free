# fyrox-ai-free

AI State Machine and Behavior Tree for [Fyrox](https://fyrox.rs) game engine.

Build enemy AI like **Patrol -> Chase -> Attack -> Patrol** without writing spaghetti code.

## Features

- **AI State Machine** - Define states and transitions with condition-based rules
- **Behavior Tree** - Sequence, Selector, Parallel, Inverter, Repeater, Wait nodes
- **Blackboard** - Shared data store for AI decision-making
- **Condition System** - Composable logic trees (And/Or/Not) with comparison operators
- **Serializable** - All data structures support serde for save/load

## Quick Start

```rust
use fyrox_ai_free::*;

let mut sm = AiStateMachine::new();
let patrol = sm.add_state(AiState::new("Patrol"));
let chase = sm.add_state(AiState::new("Chase"));
let attack = sm.add_state(AiState::new("Attack"));

sm.add_transition(AiTransition::new(
    "Spot Enemy", patrol, chase,
    ConditionNode::Leaf(Condition::is_true("enemy_visible")),
));
sm.add_transition(AiTransition::new(
    "In Range", chase, attack,
    ConditionNode::Leaf(Condition::is_true("in_attack_range")),
));
sm.set_entry_state(patrol);

// In your game loop:
let mut blackboard = Blackboard::new();
blackboard.set("enemy_visible", BlackboardValue::Bool(true));
sm.tick(dt, &mut blackboard);
println!("Current: {}", sm.current_state_name()); // "Chase"
```

## Behavior Tree Example

```rust
use fyrox_ai_free::*;

let tree = BehaviorTree::new(
    BtNode::selector("Root", vec![
        BtNode::sequence("Attack", vec![
            BtNode::condition("In Range?",
                ConditionNode::Leaf(Condition::is_true("in_attack_range"))),
            BtNode::action("Do Attack", "attack"),
        ]),
        BtNode::sequence("Chase", vec![
            BtNode::condition("Enemy Visible?",
                ConditionNode::Leaf(Condition::is_true("enemy_visible"))),
            BtNode::action("Move To Enemy", "chase"),
        ]),
        BtNode::action("Patrol", "patrol"),
    ]),
);
```

## Free vs Pro

| Feature | Free | Pro |
|---------|------|-----|
| AI State Machine | :white_check_mark: | :white_check_mark: |
| Behavior Tree | :white_check_mark: | :white_check_mark: |
| Blackboard | :white_check_mark: | :white_check_mark: |
| Condition System | :white_check_mark: | :white_check_mark: |
| Serde Serialization | :white_check_mark: | :white_check_mark: |
| Fyrox Editor Integration | :x: | :white_check_mark: |
| Visual State Graph Editor | :x: | :white_check_mark: |
| Blackboard Inspector Panel | :x: | :white_check_mark: |
| Priority Support | :x: | :white_check_mark: |

**[Get the Pro version on Gumroad](https://gumroad.com/)** <!-- TODO: Replace with actual link -->

## License

MIT
