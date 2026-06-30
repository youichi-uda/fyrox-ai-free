//! Behavior tree node types ([`BtNode`]) and their evaluation runtime ([`BtRuntime`]).

use crate::blackboard::Blackboard;
use serde::{Deserialize, Serialize};

/// Result of a behavior tree node evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BtStatus {
    /// The node finished successfully.
    #[default]
    Success,
    /// The node failed.
    Failure,
    /// The node is still running and should be ticked again next frame.
    Running,
}

/// A node in the behavior tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BtNode {
    /// Runs children in order; fails on the first child that fails, succeeds if all succeed.
    Sequence {
        /// Node name.
        name: String,
        /// Child nodes evaluated in order.
        children: Vec<BtNode>,
    },
    /// Runs children in order; succeeds on the first child that succeeds (fallback).
    Selector {
        /// Node name.
        name: String,
        /// Child nodes evaluated in order.
        children: Vec<BtNode>,
    },
    /// Inverts its child's result (`Success` <-> `Failure`; `Running` passes through).
    Inverter {
        /// Node name.
        name: String,
        /// Child node.
        child: Box<BtNode>,
    },
    /// Repeats its child until it has finished `max_count` times (0 = infinite).
    Repeater {
        /// Node name.
        name: String,
        /// Child node.
        child: Box<BtNode>,
        /// Number of completions before succeeding; `0` repeats forever.
        max_count: u32,
    },
    /// Runs its child and always reports `Success` (unless the child is `Running`).
    AlwaysSucceed {
        /// Node name.
        name: String,
        /// Child node.
        child: Box<BtNode>,
    },
    /// Succeeds if its condition evaluates to true, otherwise fails.
    ConditionCheck {
        /// Node name.
        name: String,
        /// Condition tree evaluated against the blackboard.
        condition: crate::state_machine::condition::ConditionNode,
    },
    /// A leaf whose status is supplied by the game through `action_results`.
    Action {
        /// Node name.
        name: String,
        /// Identifier the game uses to report this action's status.
        action_id: String,
    },
    /// Ticks all children every frame; succeeds when `success_threshold` children succeed.
    Parallel {
        /// Node name.
        name: String,
        /// Child nodes, all ticked each frame.
        children: Vec<BtNode>,
        /// Number of successful children required to succeed; `0` means "all".
        success_threshold: usize,
    },
    /// Reports `Running` until `duration` seconds have elapsed, then `Success`.
    Wait {
        /// Node name.
        name: String,
        /// Wait duration in seconds.
        duration: f32,
    },
}

impl Default for BtNode {
    fn default() -> Self {
        Self::Action {
            name: "Action".to_string(),
            action_id: String::new(),
        }
    }
}

/// Runtime state for behavior tree evaluation. Not serialized.
#[derive(Debug, Clone, Default)]
pub struct BtRuntime {
    node_states: Vec<NodeRuntime>,
}

#[derive(Debug, Clone, Default)]
struct NodeRuntime {
    running_index: usize,
    repeat_count: u32,
    elapsed: f32,
    last_status: BtStatus,
}

impl BtRuntime {
    /// Allocates runtime state for the tree rooted at `root`.
    pub fn new(root: &BtNode) -> Self {
        let count = count_nodes(root);
        Self {
            node_states: vec![NodeRuntime::default(); count],
        }
    }

    /// Resets all per-node runtime state to its initial value.
    pub fn reset(&mut self) {
        for s in &mut self.node_states {
            *s = NodeRuntime::default();
        }
    }
}

fn count_nodes(node: &BtNode) -> usize {
    let mut count = 1;
    match node {
        BtNode::Sequence { children, .. }
        | BtNode::Selector { children, .. }
        | BtNode::Parallel { children, .. } => {
            for child in children {
                count += count_nodes(child);
            }
        }
        BtNode::Inverter { child, .. }
        | BtNode::AlwaysSucceed { child, .. }
        | BtNode::Repeater { child, .. } => {
            count += count_nodes(child);
        }
        BtNode::ConditionCheck { .. } | BtNode::Action { .. } | BtNode::Wait { .. } => {}
    }
    count
}

/// Context passed to behavior tree evaluation.
pub struct BtContext<'a> {
    /// Blackboard read by condition nodes.
    pub blackboard: &'a Blackboard,
    /// Time delta for this tick, in seconds.
    pub dt: f32,
    /// Per-action status supplied by the game, keyed by `action_id`.
    pub action_results: &'a std::collections::HashMap<String, BtStatus>,
}

impl BtNode {
    /// Builds a [`BtNode::Sequence`].
    pub fn sequence(name: impl Into<String>, children: Vec<BtNode>) -> Self {
        Self::Sequence { name: name.into(), children }
    }

    /// Builds a [`BtNode::Selector`].
    pub fn selector(name: impl Into<String>, children: Vec<BtNode>) -> Self {
        Self::Selector { name: name.into(), children }
    }

    /// Builds a [`BtNode::Action`].
    pub fn action(name: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self::Action { name: name.into(), action_id: action_id.into() }
    }

    /// Builds a [`BtNode::ConditionCheck`].
    pub fn condition(name: impl Into<String>, condition: crate::state_machine::condition::ConditionNode) -> Self {
        Self::ConditionCheck { name: name.into(), condition }
    }

    /// Builds a [`BtNode::Wait`].
    pub fn wait(name: impl Into<String>, duration: f32) -> Self {
        Self::Wait { name: name.into(), duration }
    }

    /// Builds a [`BtNode::Inverter`].
    pub fn inverter(name: impl Into<String>, child: BtNode) -> Self {
        Self::Inverter { name: name.into(), child: Box::new(child) }
    }

    /// Builds a [`BtNode::Parallel`]. `success_threshold` of `0` means *all* children must succeed.
    pub fn parallel(
        name: impl Into<String>,
        children: Vec<BtNode>,
        success_threshold: usize,
    ) -> Self {
        Self::Parallel { name: name.into(), children, success_threshold }
    }

    /// Builds a [`BtNode::Repeater`]. `max_count` of `0` repeats forever.
    pub fn repeater(name: impl Into<String>, child: BtNode, max_count: u32) -> Self {
        Self::Repeater { name: name.into(), child: Box::new(child), max_count }
    }

    /// Builds a [`BtNode::AlwaysSucceed`] decorator.
    pub fn always_succeed(name: impl Into<String>, child: BtNode) -> Self {
        Self::AlwaysSucceed { name: name.into(), child: Box::new(child) }
    }

    /// Returns this node's name.
    pub fn name(&self) -> &str {
        match self {
            Self::Sequence { name, .. } | Self::Selector { name, .. }
            | Self::Inverter { name, .. } | Self::Repeater { name, .. }
            | Self::AlwaysSucceed { name, .. } | Self::ConditionCheck { name, .. }
            | Self::Action { name, .. } | Self::Parallel { name, .. }
            | Self::Wait { name, .. } => name,
        }
    }

    /// Evaluates this node, returning its status and the id of the next sibling node.
    ///
    /// `node_id` is the pre-order index of this node within the tree; it indexes into the
    /// [`BtRuntime`] state. Normally called indirectly via [`BehaviorTree::tick`](crate::BehaviorTree::tick).
    pub fn tick(&self, ctx: &BtContext, runtime: &mut BtRuntime, node_id: usize) -> (BtStatus, usize) {
        let mut next_id = node_id + 1;

        let status = match self {
            Self::Sequence { children, .. } => {
                let start = runtime.node_states[node_id].running_index;
                let mut skip_id = next_id;
                for child in &children[..start] {
                    skip_id += count_nodes(child);
                }
                next_id = skip_id;

                let mut result = BtStatus::Success;
                for i in start..children.len() {
                    let (child_status, after_id) = children[i].tick(ctx, runtime, next_id);
                    next_id = after_id;
                    match child_status {
                        BtStatus::Success => {
                            runtime.node_states[node_id].running_index = i + 1;
                        }
                        BtStatus::Failure => {
                            runtime.node_states[node_id].running_index = 0;
                            result = BtStatus::Failure;
                            for child in &children[i + 1..] { next_id += count_nodes(child); }
                            break;
                        }
                        BtStatus::Running => {
                            runtime.node_states[node_id].running_index = i;
                            result = BtStatus::Running;
                            for child in &children[i + 1..] { next_id += count_nodes(child); }
                            break;
                        }
                    }
                }
                if result == BtStatus::Success {
                    runtime.node_states[node_id].running_index = 0;
                }
                result
            }

            Self::Selector { children, .. } => {
                let start = runtime.node_states[node_id].running_index;
                let mut skip_id = next_id;
                for child in &children[..start] { skip_id += count_nodes(child); }
                next_id = skip_id;

                let mut result = BtStatus::Failure;
                for i in start..children.len() {
                    let (child_status, after_id) = children[i].tick(ctx, runtime, next_id);
                    next_id = after_id;
                    match child_status {
                        BtStatus::Success => {
                            runtime.node_states[node_id].running_index = 0;
                            result = BtStatus::Success;
                            for child in &children[i + 1..] { next_id += count_nodes(child); }
                            break;
                        }
                        BtStatus::Failure => {
                            runtime.node_states[node_id].running_index = i + 1;
                        }
                        BtStatus::Running => {
                            runtime.node_states[node_id].running_index = i;
                            result = BtStatus::Running;
                            for child in &children[i + 1..] { next_id += count_nodes(child); }
                            break;
                        }
                    }
                }
                if result == BtStatus::Failure {
                    runtime.node_states[node_id].running_index = 0;
                }
                result
            }

            Self::Inverter { child, .. } => {
                let (s, after_id) = child.tick(ctx, runtime, next_id);
                next_id = after_id;
                match s { BtStatus::Success => BtStatus::Failure, BtStatus::Failure => BtStatus::Success, BtStatus::Running => BtStatus::Running }
            }

            Self::Repeater { child, max_count, .. } => {
                let (s, after_id) = child.tick(ctx, runtime, next_id);
                next_id = after_id;
                if s != BtStatus::Running {
                    runtime.node_states[node_id].repeat_count += 1;
                    if *max_count > 0 && runtime.node_states[node_id].repeat_count >= *max_count {
                        runtime.node_states[node_id].repeat_count = 0;
                        s
                    } else { BtStatus::Running }
                } else { BtStatus::Running }
            }

            Self::AlwaysSucceed { child, .. } => {
                let (s, after_id) = child.tick(ctx, runtime, next_id);
                next_id = after_id;
                if s == BtStatus::Running { BtStatus::Running } else { BtStatus::Success }
            }

            Self::ConditionCheck { condition, .. } => {
                if condition.evaluate(ctx.blackboard) { BtStatus::Success } else { BtStatus::Failure }
            }

            Self::Action { action_id, .. } => {
                if let Some(&r) = ctx.action_results.get(action_id) { r }
                else { runtime.node_states[node_id].last_status }
            }

            Self::Parallel { children, success_threshold, .. } => {
                let mut success_count = 0;
                let mut running = false;
                let mut failed = false;
                for child in children {
                    let (s, after_id) = child.tick(ctx, runtime, next_id);
                    next_id = after_id;
                    match s { BtStatus::Success => success_count += 1, BtStatus::Failure => failed = true, BtStatus::Running => running = true }
                }
                let threshold = if *success_threshold == 0 { children.len() } else { *success_threshold };
                if success_count >= threshold { BtStatus::Success }
                else if failed { BtStatus::Failure }
                else if running { BtStatus::Running }
                else { BtStatus::Failure }
            }

            Self::Wait { duration, .. } => {
                runtime.node_states[node_id].elapsed += ctx.dt;
                if runtime.node_states[node_id].elapsed >= *duration {
                    runtime.node_states[node_id].elapsed = 0.0;
                    BtStatus::Success
                } else { BtStatus::Running }
            }
        };

        runtime.node_states[node_id].last_status = status;
        (status, next_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_repeater_always_succeed_builders() {
        let p = BtNode::parallel("P", vec![BtNode::action("a", "id_a")], 1);
        match p {
            BtNode::Parallel { name, children, success_threshold } => {
                assert_eq!(name, "P");
                assert_eq!(children.len(), 1);
                assert_eq!(success_threshold, 1);
            }
            _ => panic!("expected Parallel"),
        }

        let r = BtNode::repeater("R", BtNode::action("a", "id_a"), 3);
        match r {
            BtNode::Repeater { name, max_count, .. } => {
                assert_eq!(name, "R");
                assert_eq!(max_count, 3);
            }
            _ => panic!("expected Repeater"),
        }

        let s = BtNode::always_succeed("AS", BtNode::action("a", "id_a"));
        match s {
            BtNode::AlwaysSucceed { name, .. } => assert_eq!(name, "AS"),
            _ => panic!("expected AlwaysSucceed"),
        }
    }
}
