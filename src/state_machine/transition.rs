//! Transitions ([`AiTransition`]) connecting two states with a firing condition.

use super::condition::ConditionNode;
use serde::{Deserialize, Serialize};

/// A transition between two AI states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTransition {
    /// Human-readable transition name.
    pub name: String,
    /// Source state index.
    pub from: usize,
    /// Destination state index.
    pub to: usize,
    /// Condition tree that must evaluate to true for this transition to fire.
    pub condition: ConditionNode,
    /// Minimum time (seconds) the source state must be active before this transition can fire.
    pub min_time_in_state: f32,
}

impl Default for AiTransition {
    fn default() -> Self {
        Self {
            name: "Transition".to_string(),
            from: usize::MAX,
            to: usize::MAX,
            condition: ConditionNode::default(),
            min_time_in_state: 0.0,
        }
    }
}

impl AiTransition {
    /// Creates a transition from state index `from` to `to` that fires when `condition` is true.
    pub fn new(
        name: impl Into<String>,
        from: usize,
        to: usize,
        condition: ConditionNode,
    ) -> Self {
        Self {
            name: name.into(),
            from,
            to,
            condition,
            min_time_in_state: 0.0,
        }
    }

    /// Sets the minimum time the source state must be active before this transition can fire.
    pub fn with_min_time(mut self, seconds: f32) -> Self {
        self.min_time_in_state = seconds;
        self
    }
}
