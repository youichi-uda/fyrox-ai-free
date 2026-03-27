use crate::blackboard::Blackboard;
use serde::{Deserialize, Serialize};

/// Result of a behavior tree node evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BtStatus {
    #[default]
    Success,
    Failure,
    Running,
}

/// A node in the behavior tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BtNode {
    Sequence { name: String, children: Vec<BtNode> },
    Selector { name: String, children: Vec<BtNode> },
    Inverter { name: String, child: Box<BtNode> },
    Repeater { name: String, child: Box<BtNode>, max_count: u32 },
    AlwaysSucceed { name: String, child: Box<BtNode> },
    ConditionCheck { name: String, condition: crate::state_machine::condition::ConditionNode },
    Action { name: String, action_id: String },
    Parallel { name: String, children: Vec<BtNode>, success_threshold: usize },
    Wait { name: String, duration: f32 },
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
    pub fn new(root: &BtNode) -> Self {
        let count = count_nodes(root);
        Self {
            node_states: vec![NodeRuntime::default(); count],
        }
    }

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
    pub blackboard: &'a Blackboard,
    pub dt: f32,
    pub action_results: &'a std::collections::HashMap<String, BtStatus>,
}

impl BtNode {
    pub fn sequence(name: impl Into<String>, children: Vec<BtNode>) -> Self {
        Self::Sequence { name: name.into(), children }
    }

    pub fn selector(name: impl Into<String>, children: Vec<BtNode>) -> Self {
        Self::Selector { name: name.into(), children }
    }

    pub fn action(name: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self::Action { name: name.into(), action_id: action_id.into() }
    }

    pub fn condition(name: impl Into<String>, condition: crate::state_machine::condition::ConditionNode) -> Self {
        Self::ConditionCheck { name: name.into(), condition }
    }

    pub fn wait(name: impl Into<String>, duration: f32) -> Self {
        Self::Wait { name: name.into(), duration }
    }

    pub fn inverter(name: impl Into<String>, child: BtNode) -> Self {
        Self::Inverter { name: name.into(), child: Box::new(child) }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Sequence { name, .. } | Self::Selector { name, .. }
            | Self::Inverter { name, .. } | Self::Repeater { name, .. }
            | Self::AlwaysSucceed { name, .. } | Self::ConditionCheck { name, .. }
            | Self::Action { name, .. } | Self::Parallel { name, .. }
            | Self::Wait { name, .. } => name,
        }
    }

    pub fn tick(&self, ctx: &BtContext, runtime: &mut BtRuntime, node_id: usize) -> (BtStatus, usize) {
        let mut next_id = node_id + 1;

        let status = match self {
            Self::Sequence { children, .. } => {
                let start = runtime.node_states[node_id].running_index;
                let mut skip_id = next_id;
                for i in 0..start {
                    skip_id += count_nodes(&children[i]);
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
                            for j in (i + 1)..children.len() { next_id += count_nodes(&children[j]); }
                            break;
                        }
                        BtStatus::Running => {
                            runtime.node_states[node_id].running_index = i;
                            result = BtStatus::Running;
                            for j in (i + 1)..children.len() { next_id += count_nodes(&children[j]); }
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
                for i in 0..start { skip_id += count_nodes(&children[i]); }
                next_id = skip_id;

                let mut result = BtStatus::Failure;
                for i in start..children.len() {
                    let (child_status, after_id) = children[i].tick(ctx, runtime, next_id);
                    next_id = after_id;
                    match child_status {
                        BtStatus::Success => {
                            runtime.node_states[node_id].running_index = 0;
                            result = BtStatus::Success;
                            for j in (i + 1)..children.len() { next_id += count_nodes(&children[j]); }
                            break;
                        }
                        BtStatus::Failure => {
                            runtime.node_states[node_id].running_index = i + 1;
                        }
                        BtStatus::Running => {
                            runtime.node_states[node_id].running_index = i;
                            result = BtStatus::Running;
                            for j in (i + 1)..children.len() { next_id += count_nodes(&children[j]); }
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
