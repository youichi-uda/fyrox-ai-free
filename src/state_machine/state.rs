//! State definitions ([`AiState`]) and the actions ([`StateAction`]) they run on enter/leave.

use serde::{Deserialize, Serialize};

/// An action to execute when entering or leaving a state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum StateAction {
    /// Do nothing.
    #[default]
    None,
    /// Set a blackboard value.
    SetBlackboard {
        /// Blackboard key to write to.
        key: String,
        /// Value to store.
        value: crate::blackboard::BlackboardValue,
    },
}

/// A single state in the [`AiStateMachine`](crate::AiStateMachine).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiState {
    /// Human-readable state name.
    pub name: String,
    /// Actions to execute when entering this state.
    pub on_enter_actions: Vec<StateAction>,
    /// Actions to execute when leaving this state.
    pub on_leave_actions: Vec<StateAction>,
    /// Indices of transitions originating from this state.
    pub transitions: Vec<usize>,
    /// Position in the editor graph view (for visual editing).
    pub position: [f32; 2],
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            name: "New State".to_string(),
            on_enter_actions: Vec::new(),
            on_leave_actions: Vec::new(),
            transitions: Vec::new(),
            position: [0.0, 0.0],
        }
    }
}

impl AiState {
    /// Creates a new state with the given name and no actions or transitions.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Sets the editor graph position and returns `self` (builder style).
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = [x, y];
        self
    }

    /// Adds an on-enter action and returns `self` (builder style).
    pub fn with_on_enter(mut self, action: StateAction) -> Self {
        self.on_enter_actions.push(action);
        self
    }

    /// Adds an on-leave action and returns `self` (builder style).
    pub fn with_on_leave(mut self, action: StateAction) -> Self {
        self.on_leave_actions.push(action);
        self
    }

    /// Runs every on-enter action against `blackboard`.
    pub fn execute_enter_actions(&self, blackboard: &mut crate::blackboard::Blackboard) {
        for action in &self.on_enter_actions {
            execute_action(action, blackboard);
        }
    }

    /// Runs every on-leave action against `blackboard`.
    pub fn execute_leave_actions(&self, blackboard: &mut crate::blackboard::Blackboard) {
        for action in &self.on_leave_actions {
            execute_action(action, blackboard);
        }
    }
}

fn execute_action(action: &StateAction, blackboard: &mut crate::blackboard::Blackboard) {
    match action {
        StateAction::None => {}
        StateAction::SetBlackboard { key, value } => {
            blackboard.set(key.clone(), value.clone());
        }
    }
}
