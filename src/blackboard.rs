//! Shared key/value data store ([`Blackboard`]) used by the state machine and behavior tree.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A value that can be stored in the [`Blackboard`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlackboardValue {
    /// Boolean value.
    Bool(bool),
    /// 64-bit signed integer value.
    Int(i64),
    /// 32-bit floating point value.
    Float(f32),
    /// UTF-8 string value.
    String(String),
    /// 3D vector value.
    Vec3 {
        /// X component.
        x: f32,
        /// Y component.
        y: f32,
        /// Z component.
        z: f32,
    },
}

impl Default for BlackboardValue {
    fn default() -> Self {
        Self::Bool(false)
    }
}

impl BlackboardValue {
    /// Returns the inner value if this is a [`BlackboardValue::Bool`], otherwise `None`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the inner value if this is a [`BlackboardValue::Int`], otherwise `None`.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the inner value if this is a [`BlackboardValue::Float`], otherwise `None`.
    pub fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the inner value if this is a [`BlackboardValue::String`], otherwise `None`.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }
}

/// Shared data store for AI decision-making. Scripts write sensor data here,
/// and the state machine / behavior tree reads from it to make decisions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Blackboard {
    entries: HashMap<String, BlackboardValue>,
}

impl Blackboard {
    /// Creates a new, empty blackboard.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or overwrites the value stored under `key`.
    pub fn set(&mut self, key: impl Into<String>, value: BlackboardValue) {
        self.entries.insert(key.into(), value);
    }

    /// Returns a reference to the value stored under `key`, if any.
    pub fn get(&self, key: &str) -> Option<&BlackboardValue> {
        self.entries.get(key)
    }

    /// Returns the boolean stored under `key`, if it exists and is a [`BlackboardValue::Bool`].
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    /// Returns the integer stored under `key`, if it exists and is a [`BlackboardValue::Int`].
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_int())
    }

    /// Returns the float stored under `key`, if it exists and is a [`BlackboardValue::Float`].
    pub fn get_float(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|v| v.as_float())
    }

    /// Returns the string stored under `key`, if it exists and is a [`BlackboardValue::String`].
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_string())
    }

    /// Removes the value stored under `key`, returning it if it existed.
    pub fn remove(&mut self, key: &str) -> Option<BlackboardValue> {
        self.entries.remove(key)
    }

    /// Returns `true` if the blackboard contains an entry for `key`.
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Removes all entries from the blackboard.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns an iterator over all `(key, value)` pairs. Order is unspecified.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &BlackboardValue)> {
        self.entries.iter()
    }
}
