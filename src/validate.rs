//! Static validation for [`AiStateMachine`] and [`BehaviorTree`] graphs.
//!
//! These checks catch common authoring mistakes (dangling transitions, missing entry
//! state, unreachable states, empty action ids) without running the simulation.
//!
//! ```
//! use fyrox_ai_free::*;
//!
//! let mut sm = AiStateMachine::new();
//! let a = sm.add_state(AiState::new("A"));
//! sm.set_entry_state(a);
//! assert!(sm.validate().is_ok());
//! ```

use crate::behavior_tree::{node::BtNode, BehaviorTree};
use crate::state_machine::AiStateMachine;
use std::collections::HashSet;

/// A problem reported by [`AiStateMachine::validate`] or [`BehaviorTree::validate`].
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// A transition points at an out-of-range state index.
    ///
    /// `from` / `to` are the indices recorded on the transition; `max` is the number
    /// of states currently in the machine (so any valid index must be `< max`).
    DanglingTransition {
        /// Source state index recorded on the transition.
        from: usize,
        /// Destination state index recorded on the transition.
        to: usize,
        /// Number of states in the machine; valid indices are `< max`.
        max: usize,
    },
    /// The state machine has no entry state set.
    MissingEntryState,
    /// A state is not reachable from the entry state via any sequence of transitions.
    UnreachableState(usize),
    /// A [`BtNode::Action`] node has an empty `action_id` (its node name is reported).
    EmptyActionId(String),
}

impl AiStateMachine {
    /// Static checks: entry state present, every transition's `from`/`to` in range,
    /// every state reachable from the entry state via BFS.
    ///
    /// Returns `Ok(())` if the machine is well-formed, otherwise a `Vec` collecting
    /// every problem found (the function does not short-circuit on the first error).
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let n_states = self.states().len();

        // Range-check every transition.
        for t in self.transitions() {
            if t.from >= n_states || t.to >= n_states {
                errors.push(ValidationError::DanglingTransition {
                    from: t.from,
                    to: t.to,
                    max: n_states,
                });
            }
        }

        // Entry state presence + reachability BFS (only meaningful if there are states).
        match self.entry_state() {
            None if n_states > 0 => errors.push(ValidationError::MissingEntryState),
            None => {}
            Some(entry) => {
                if entry >= n_states {
                    errors.push(ValidationError::DanglingTransition {
                        from: entry,
                        to: entry,
                        max: n_states,
                    });
                } else {
                    let mut visited = HashSet::new();
                    let mut stack = vec![entry];
                    while let Some(s) = stack.pop() {
                        if !visited.insert(s) {
                            continue;
                        }
                        if let Some(state) = self.state(s) {
                            for &t_idx in &state.transitions {
                                if let Some(t) = self.transition(t_idx) {
                                    if t.to < n_states && !visited.contains(&t.to) {
                                        stack.push(t.to);
                                    }
                                }
                            }
                        }
                    }
                    for i in 0..n_states {
                        if !visited.contains(&i) {
                            errors.push(ValidationError::UnreachableState(i));
                        }
                    }
                }
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

impl BehaviorTree {
    /// Walks the tree and reports every [`BtNode::Action`] leaf with an empty `action_id`.
    ///
    /// Returns `Ok(())` if no problems are found.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        walk_bt(self.root(), &mut errors);
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

fn walk_bt(node: &BtNode, errors: &mut Vec<ValidationError>) {
    match node {
        BtNode::Action { name, action_id } => {
            if action_id.trim().is_empty() {
                errors.push(ValidationError::EmptyActionId(name.clone()));
            }
        }
        BtNode::Sequence { children, .. }
        | BtNode::Selector { children, .. }
        | BtNode::Parallel { children, .. } => {
            for c in children {
                walk_bt(c, errors);
            }
        }
        BtNode::Inverter { child, .. }
        | BtNode::AlwaysSucceed { child, .. }
        | BtNode::Repeater { child, .. } => {
            walk_bt(child, errors);
        }
        BtNode::ConditionCheck { .. } | BtNode::Wait { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blackboard::BlackboardValue;
    use crate::state_machine::{
        condition::{Condition, ConditionNode},
        state::AiState,
        transition::AiTransition,
    };

    #[test]
    fn statemachine_ok_when_well_formed() {
        let mut sm = AiStateMachine::new();
        let a = sm.add_state(AiState::new("A"));
        let b = sm.add_state(AiState::new("B"));
        sm.add_transition(AiTransition::new(
            "A->B",
            a,
            b,
            ConditionNode::Leaf(Condition::is_true("go")),
        ));
        sm.set_entry_state(a);
        assert!(sm.validate().is_ok());
    }

    #[test]
    fn statemachine_detects_missing_entry_and_dangling_and_unreachable() {
        let mut sm = AiStateMachine::new();
        let _a = sm.add_state(AiState::new("A"));
        let _b = sm.add_state(AiState::new("B"));
        // dangling: `to` points past the end
        sm.add_transition(AiTransition::new(
            "bad",
            0,
            99,
            ConditionNode::Leaf(Condition::is_true("x")),
        ));
        // no entry state set

        let errs = sm.validate().unwrap_err();
        assert!(errs.iter().any(|e| matches!(
            e,
            ValidationError::DanglingTransition { to: 99, max: 2, .. }
        )));
        assert!(errs.contains(&ValidationError::MissingEntryState));

        // Now set entry — B becomes unreachable (only the dangling transition references it,
        // and 99 is out of range so BFS cannot reach 1).
        sm.set_entry_state(0);
        let errs = sm.validate().unwrap_err();
        assert!(errs.contains(&ValidationError::UnreachableState(1)));
    }

    #[test]
    fn behaviortree_detects_empty_action_id() {
        let tree = BehaviorTree::new(BtNode::sequence(
            "root",
            vec![
                BtNode::action("named", "ok_id"),
                BtNode::action("empty", ""),
                BtNode::condition(
                    "c",
                    ConditionNode::Leaf(Condition::eq("k", BlackboardValue::Bool(true))),
                ),
            ],
        ));
        let errs = tree.validate().unwrap_err();
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0], ValidationError::EmptyActionId("empty".into()));

        let good = BehaviorTree::new(BtNode::action("only", "id"));
        assert!(good.validate().is_ok());
    }
}
