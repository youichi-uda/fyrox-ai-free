pub mod node;

use crate::blackboard::Blackboard;
use node::{BtContext, BtNode, BtRuntime, BtStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A behavior tree that drives AI decision-making.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BehaviorTree {
    root: BtNode,
    active: bool,
}

impl BehaviorTree {
    pub fn new(root: BtNode) -> Self {
        Self { root, active: true }
    }

    pub fn root(&self) -> &BtNode { &self.root }
    pub fn root_mut(&mut self) -> &mut BtNode { &mut self.root }
    pub fn is_active(&self) -> bool { self.active }
    pub fn set_active(&mut self, active: bool) { self.active = active; }

    pub fn create_runtime(&self) -> BtRuntime {
        BtRuntime::new(&self.root)
    }

    pub fn tick(
        &self, dt: f32, blackboard: &Blackboard,
        action_results: &HashMap<String, BtStatus>, runtime: &mut BtRuntime,
    ) -> BtStatus {
        if !self.active { return BtStatus::Failure; }
        let ctx = BtContext { blackboard, dt, action_results };
        let (status, _) = self.root.tick(&ctx, runtime, 0);
        status
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blackboard::BlackboardValue;
    use crate::state_machine::condition::{Condition, ConditionNode};

    #[test]
    fn test_selector_fallback() {
        let bb = &mut Blackboard::new();
        let actions = &mut HashMap::new();

        let tree = BehaviorTree::new(BtNode::selector("Root", vec![
            BtNode::sequence("Attack", vec![
                BtNode::condition("In Range?", ConditionNode::Leaf(Condition::is_true("in_range"))),
                BtNode::action("Do Attack", "attack"),
            ]),
            BtNode::action("Patrol", "patrol"),
        ]));
        let mut runtime = tree.create_runtime();

        bb.set("in_range", BlackboardValue::Bool(false));
        actions.insert("patrol".to_string(), BtStatus::Success);
        actions.insert("attack".to_string(), BtStatus::Success);
        assert_eq!(tree.tick(0.016, bb, actions, &mut runtime), BtStatus::Success);

        runtime.reset();
        bb.set("in_range", BlackboardValue::Bool(true));
        assert_eq!(tree.tick(0.016, bb, actions, &mut runtime), BtStatus::Success);
    }

    #[test]
    fn test_wait_node() {
        let bb = Blackboard::new();
        let actions = HashMap::new();
        let tree = BehaviorTree::new(BtNode::wait("Wait 1s", 1.0));
        let mut runtime = tree.create_runtime();

        assert_eq!(tree.tick(0.3, &bb, &actions, &mut runtime), BtStatus::Running);
        assert_eq!(tree.tick(0.3, &bb, &actions, &mut runtime), BtStatus::Running);
        assert_eq!(tree.tick(0.5, &bb, &actions, &mut runtime), BtStatus::Success);
    }
}
