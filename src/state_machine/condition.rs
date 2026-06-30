//! Condition system: [`Condition`] leaves combined into [`ConditionNode`] logic trees.

use crate::blackboard::{Blackboard, BlackboardValue};
use serde::{Deserialize, Serialize};

/// Comparison operator for condition evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CompareOp {
    /// `actual == expected`.
    #[default]
    Equal,
    /// `actual != expected`.
    NotEqual,
    /// `actual < expected`.
    Less,
    /// `actual <= expected`.
    LessOrEqual,
    /// `actual > expected`.
    Greater,
    /// `actual >= expected`.
    GreaterOrEqual,
}

/// A single condition that compares a blackboard value against an expected value.
///
/// # Floating point comparisons
///
/// For [`BlackboardValue::Float`], [`CompareOp::Equal`] and [`CompareOp::NotEqual`] use an
/// approximate comparison with a tolerance of [`Condition::float_tolerance`] (default
/// [`f32::EPSILON`], i.e. *near-exact*). Because game values such as health or distance are
/// rarely exactly equal, prefer [`CompareOp::Greater`] / [`CompareOp::Less`] for floats, or
/// widen the tolerance via [`Condition::with_tolerance`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Blackboard key to read the actual value from.
    pub key: String,
    /// Comparison operator.
    pub op: CompareOp,
    /// Expected value to compare against.
    pub value: BlackboardValue,
    /// Tolerance used for approximate float equality. See the type-level docs.
    #[serde(default = "Condition::default_tolerance")]
    pub float_tolerance: f32,
}

impl Default for Condition {
    fn default() -> Self {
        Self {
            key: String::new(),
            op: CompareOp::Equal,
            value: BlackboardValue::Bool(true),
            float_tolerance: Self::default_tolerance(),
        }
    }
}

impl Condition {
    fn default_tolerance() -> f32 {
        f32::EPSILON
    }

    /// The default float equality tolerance ([`f32::EPSILON`]).
    pub const fn float_tolerance() -> f32 {
        f32::EPSILON
    }

    /// Creates a condition comparing the blackboard value at `key` against `value` using `op`.
    pub fn new(key: impl Into<String>, op: CompareOp, value: BlackboardValue) -> Self {
        Self {
            key: key.into(),
            op,
            value,
            float_tolerance: Self::default_tolerance(),
        }
    }

    /// Sets the tolerance used for approximate float `Equal`/`NotEqual` comparisons.
    pub fn with_tolerance(mut self, tolerance: f32) -> Self {
        self.float_tolerance = tolerance;
        self
    }

    /// Shorthand for `key == true`.
    pub fn is_true(key: impl Into<String>) -> Self {
        Self::new(key, CompareOp::Equal, BlackboardValue::Bool(true))
    }

    /// Shorthand for `key == false`.
    pub fn is_false(key: impl Into<String>) -> Self {
        Self::new(key, CompareOp::Equal, BlackboardValue::Bool(false))
    }

    /// Evaluates this condition against `blackboard`. Returns `false` if the key is missing or
    /// the stored value has a different type than the expected value.
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
                compare_op_f32(self.op, *actual, *expected, self.float_tolerance)
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

fn compare_op_f32(op: CompareOp, actual: f32, expected: f32, tolerance: f32) -> bool {
    match op {
        CompareOp::Equal => (actual - expected).abs() < tolerance,
        CompareOp::NotEqual => (actual - expected).abs() >= tolerance,
        CompareOp::Less => actual < expected,
        CompareOp::LessOrEqual => actual <= expected,
        CompareOp::Greater => actual > expected,
        CompareOp::GreaterOrEqual => actual >= expected,
    }
}

/// A logic tree that combines multiple [`Condition`]s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionNode {
    /// Single condition check.
    Leaf(Condition),
    /// All children must be true (logical AND). An empty list is `true`.
    And(Vec<ConditionNode>),
    /// At least one child must be true (logical OR). An empty list is `false`.
    Or(Vec<ConditionNode>),
    /// Inverts the child result (logical NOT).
    Not(Box<ConditionNode>),
}

impl Default for ConditionNode {
    fn default() -> Self {
        Self::Leaf(Condition::default())
    }
}

impl ConditionNode {
    /// Recursively evaluates this logic tree against `blackboard`.
    pub fn evaluate(&self, blackboard: &Blackboard) -> bool {
        match self {
            Self::Leaf(condition) => condition.evaluate(blackboard),
            Self::And(children) => children.iter().all(|c| c.evaluate(blackboard)),
            Self::Or(children) => children.iter().any(|c| c.evaluate(blackboard)),
            Self::Not(child) => !child.evaluate(blackboard),
        }
    }
}
