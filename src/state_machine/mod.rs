pub mod condition;
pub mod state;
pub mod transition;

use crate::blackboard::Blackboard;
use serde::{Deserialize, Serialize};
use state::AiState;
use transition::AiTransition;

/// The result of evaluating the state machine for one tick.
#[derive(Debug, Clone)]
pub struct AiStateMachineEvent {
    pub prev_state: usize,
    pub new_state: usize,
    pub transition: usize,
}

/// AI State Machine — manages states and transitions driven by a blackboard.
///
/// # Example: Enemy Patrol AI
/// ```
/// use fyrox_ai_free::*;
///
/// let mut sm = AiStateMachine::new();
/// let patrol = sm.add_state(AiState::new("Patrol"));
/// let chase = sm.add_state(AiState::new("Chase"));
/// let attack = sm.add_state(AiState::new("Attack"));
///
/// sm.add_transition(AiTransition::new(
///     "Spot Enemy",
///     patrol, chase,
///     ConditionNode::Leaf(Condition::is_true("enemy_visible")),
/// ));
/// sm.add_transition(AiTransition::new(
///     "In Range",
///     chase, attack,
///     ConditionNode::Leaf(Condition::is_true("in_attack_range")),
/// ));
/// sm.add_transition(AiTransition::new(
///     "Lost Enemy",
///     chase, patrol,
///     ConditionNode::Leaf(Condition::is_false("enemy_visible")),
/// ));
/// sm.set_entry_state(patrol);
///
/// let mut bb = Blackboard::new();
/// bb.set("enemy_visible", BlackboardValue::Bool(true));
/// let event = sm.tick(0.016, &mut bb);
/// assert!(event.is_some());
/// assert_eq!(sm.current_state_name(), "Chase");
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiStateMachine {
    states: Vec<AiState>,
    transitions: Vec<AiTransition>,
    entry_state: Option<usize>,
    current_state: Option<usize>,
    time_in_current_state: f32,
    active: bool,
}

impl AiStateMachine {
    pub fn new() -> Self {
        Self {
            active: true,
            ..Default::default()
        }
    }

    /// Add a state and return its index.
    pub fn add_state(&mut self, state: AiState) -> usize {
        let idx = self.states.len();
        self.states.push(state);
        idx
    }

    /// Add a transition and return its index.
    pub fn add_transition(&mut self, transition: AiTransition) -> usize {
        let from = transition.from;
        let idx = self.transitions.len();
        self.transitions.push(transition);
        if let Some(state) = self.states.get_mut(from) {
            state.transitions.push(idx);
        }
        idx
    }

    pub fn set_entry_state(&mut self, state: usize) {
        self.entry_state = Some(state);
        if self.current_state.is_none() {
            self.current_state = Some(state);
        }
    }

    pub fn entry_state(&self) -> Option<usize> {
        self.entry_state
    }

    pub fn current_state(&self) -> Option<usize> {
        self.current_state
    }

    pub fn current_state_ref(&self) -> Option<&AiState> {
        self.current_state.and_then(|i| self.states.get(i))
    }

    pub fn current_state_name(&self) -> &str {
        self.current_state_ref()
            .map(|s| s.name.as_str())
            .unwrap_or("<none>")
    }

    pub fn time_in_current_state(&self) -> f32 {
        self.time_in_current_state
    }

    pub fn states(&self) -> &[AiState] {
        &self.states
    }

    pub fn states_mut(&mut self) -> &mut Vec<AiState> {
        &mut self.states
    }

    pub fn transitions(&self) -> &[AiTransition] {
        &self.transitions
    }

    pub fn state(&self, index: usize) -> Option<&AiState> {
        self.states.get(index)
    }

    pub fn state_mut(&mut self, index: usize) -> Option<&mut AiState> {
        self.states.get_mut(index)
    }

    pub fn transition(&self, index: usize) -> Option<&AiTransition> {
        self.transitions.get(index)
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Force-set the current state.
    pub fn force_state(&mut self, state: usize, blackboard: &mut Blackboard) {
        if self.current_state != Some(state) {
            if let Some(old) = self.current_state.and_then(|i| self.states.get(i)) {
                old.execute_leave_actions(blackboard);
            }
            self.current_state = Some(state);
            self.time_in_current_state = 0.0;
            if let Some(new) = self.states.get(state) {
                new.execute_enter_actions(blackboard);
            }
        }
    }

    /// Evaluate the state machine for one tick. Returns a transition event if a state change occurred.
    pub fn tick(&mut self, dt: f32, blackboard: &mut Blackboard) -> Option<AiStateMachineEvent> {
        let current = self.current_state?;
        if !self.active {
            return None;
        }

        self.time_in_current_state += dt;

        let transition_indices: Vec<usize> = self
            .states
            .get(current)
            .map(|s| s.transitions.clone())
            .unwrap_or_default();

        for &t_idx in &transition_indices {
            let Some(transition) = self.transitions.get(t_idx) else {
                continue;
            };

            if self.time_in_current_state < transition.min_time_in_state {
                continue;
            }

            if transition.condition.evaluate(blackboard) {
                let next = transition.to;

                if let Some(old_state) = self.states.get(current) {
                    old_state.execute_leave_actions(blackboard);
                }

                self.current_state = Some(next);
                self.time_in_current_state = 0.0;

                if let Some(new_state) = self.states.get(next) {
                    new_state.execute_enter_actions(blackboard);
                }

                return Some(AiStateMachineEvent {
                    prev_state: current,
                    new_state: next,
                    transition: t_idx,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blackboard::BlackboardValue;
    use condition::{Condition, ConditionNode};

    #[test]
    fn test_basic_patrol_chase_flow() {
        let mut sm = AiStateMachine::new();
        let mut bb = Blackboard::new();

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

        sm.set_entry_state(patrol);
        assert_eq!(sm.current_state_name(), "Patrol");

        bb.set("enemy_visible", BlackboardValue::Bool(false));
        assert!(sm.tick(0.016, &mut bb).is_none());
        assert_eq!(sm.current_state_name(), "Patrol");

        bb.set("enemy_visible", BlackboardValue::Bool(true));
        let event = sm.tick(0.016, &mut bb);
        assert!(event.is_some());
        assert_eq!(sm.current_state_name(), "Chase");

        bb.set("in_attack_range", BlackboardValue::Bool(true));
        let event = sm.tick(0.016, &mut bb);
        assert!(event.is_some());
        assert_eq!(sm.current_state_name(), "Attack");
    }

    #[test]
    fn test_min_time_in_state() {
        let mut sm = AiStateMachine::new();
        let mut bb = Blackboard::new();

        let a = sm.add_state(AiState::new("A"));
        let b = sm.add_state(AiState::new("B"));

        sm.add_transition(
            AiTransition::new("A->B", a, b, ConditionNode::Leaf(Condition::is_true("go")))
                .with_min_time(1.0),
        );
        sm.set_entry_state(a);

        bb.set("go", BlackboardValue::Bool(true));
        assert!(sm.tick(0.5, &mut bb).is_none());
        assert_eq!(sm.current_state_name(), "A");

        assert!(sm.tick(0.6, &mut bb).is_some());
        assert_eq!(sm.current_state_name(), "B");
    }
}
