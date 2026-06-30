//! # fyrox-ai-free
//!
//! AI building blocks for the [Fyrox](https://fyrox.rs) game engine: a condition-driven
//! **state machine**, a **behavior tree**, and a shared **blackboard** data store.
//!
//! This is the free (MIT-licensed) core. A commercial **Pro** edition adds a visual
//! editor plugin, a transition-history debug logger and a priority-based state machine —
//! see <https://y1uda.itch.io/fyrox-ai-pro>.
//!
//! ## Quick start
//!
//! ```
//! use fyrox_ai_free::*;
//!
//! let mut sm = AiStateMachine::new();
//! let patrol = sm.add_state(AiState::new("Patrol"));
//! let chase = sm.add_state(AiState::new("Chase"));
//! sm.add_transition(AiTransition::new(
//!     "Spot Enemy",
//!     patrol,
//!     chase,
//!     ConditionNode::Leaf(Condition::is_true("enemy_visible")),
//! ));
//! sm.set_entry_state(patrol);
//!
//! let mut bb = Blackboard::new();
//! bb.set("enemy_visible", BlackboardValue::Bool(true));
//! sm.tick(0.016, &mut bb);
//! assert_eq!(sm.current_state_name(), "Chase");
//! ```
#![warn(missing_docs)]

pub mod behavior_tree;
pub mod blackboard;
pub mod state_machine;
pub mod validate;

pub use behavior_tree::{
    node::{BtContext, BtNode, BtRuntime, BtStatus},
    BehaviorTree,
};
pub use blackboard::{Blackboard, BlackboardValue};
pub use state_machine::{
    condition::{CompareOp, Condition, ConditionNode},
    state::{AiState, StateAction},
    transition::AiTransition,
    AiStateMachine, AiStateMachineEvent,
};
pub use validate::ValidationError;
