use crate::blackboard::{Blackboard, BlackboardValue};
use serde::{Deserialize, Serialize};

/// Comparison operator for condition evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

impl Default for CompareOp {
    fn default() -> Self {
        Self::Equal
    }
}

/// A single condition that checks a blackboard value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub key: String,
    pub op: CompareOp,
    pub value: BlackboardValue,
}

impl Default for Condition {
    fn default() -> Self {
        Self {
            key: String::new(),
            op: CompareOp::Equal,
            value: BlackboardValue::Bool(true),
        }
    }
}

impl Condition {
    pub fn new(key: impl Into<String>, op: CompareOp, value: BlackboardValue) -> Self {
        Self {
            key: key.into(),
            op,
            value,
        }
    }

    /// Shorthand: `key == true`
    pub fn is_true(key: impl Into<String>) -> Self {
        Self::new(key, CompareOp::Equal, BlackboardValue::Bool(true))
    }

    /// Shorthand: `key == false`
    pub fn is_false(key: impl Into<String>) -> Self {
        Self::new(key, CompareOp::Equal, BlackboardValue::Bool(false))
    }

    pub fn evaluate(&self, blackboard: &Blackboard) -> bool {
        let Some(actual) = blackboard.get(&self.key) else {
            return false;
        };

        match (&self.value, actual) {
            (BlackboardValue::Bool(expected), BlackboardValue::Bool(actual)) => {
                compare_op(self.op, *actual as i64, *expected as i64)
            }
            (BlackboardValue::Int(expected), BlackboardValue::Int(actual)) => {
                compare_op(self.op, *actual, *expected)
            }
            (BlackboardValue::Float(expected), BlackboardValue::Float(actual)) => {
                compare_op_f32(self.op, *actual, *expected)
            }
            (BlackboardValue::String(expected), BlackboardValue::String(actual)) => match self.op {
                CompareOp::Equal => actual == expected,
                CompareOp::NotEqual => actual != expected,
                _ => false,
            },
            _ => false,
        }
    }
}

fn compare_op(op: CompareOp, actual: i64, expected: i64) -> bool {
    match op {
        CompareOp::Equal => actual == expected,
        CompareOp::NotEqual => actual != expected,
        CompareOp::Less => actual < expected,
        CompareOp::LessOrEqual => actual <= expected,
        CompareOp::Greater => actual > expected,
        CompareOp::GreaterOrEqual => actual >= expected,
    }
}

fn compare_op_f32(op: CompareOp, actual: f32, expected: f32) -> bool {
    match op {
        CompareOp::Equal => (actual - expected).abs() < f32::EPSILON,
        CompareOp::NotEqual => (actual - expected).abs() >= f32::EPSILON,
        CompareOp::Less => actual < expected,
        CompareOp::LessOrEqual => actual <= expected,
        CompareOp::Greater => actual > expected,
        CompareOp::GreaterOrEqual => actual >= expected,
    }
}

/// A logic tree that combines multiple conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionNode {
    /// Single condition check.
    Leaf(Condition),
    /// All children must be true.
    And(Vec<ConditionNode>),
    /// At least one child must be true.
    Or(Vec<ConditionNode>),
    /// Inverts the child result.
    Not(Box<ConditionNode>),
}

impl Default for ConditionNode {
    fn default() -> Self {
        Self::Leaf(Condition::default())
    }
}

impl ConditionNode {
    pub fn evaluate(&self, blackboard: &Blackboard) -> bool {
        match self {
            Self::Leaf(condition) => condition.evaluate(blackboard),
            Self::And(children) => children.iter().all(|c| c.evaluate(blackboard)),
            Self::Or(children) => children.iter().any(|c| c.evaluate(blackboard)),
            Self::Not(child) => !child.evaluate(blackboard),
        }
    }
}
