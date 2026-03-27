use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A value that can be stored in the blackboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlackboardValue {
    Bool(bool),
    Int(i64),
    Float(f32),
    String(String),
    Vec3 { x: f32, y: f32, z: f32 },
}

impl Default for BlackboardValue {
    fn default() -> Self {
        Self::Bool(false)
    }
}

impl BlackboardValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: impl Into<String>, value: BlackboardValue) {
        self.entries.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&BlackboardValue> {
        self.entries.get(key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_int())
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|v| v.as_float())
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_string())
    }

    pub fn remove(&mut self, key: &str) -> Option<BlackboardValue> {
        self.entries.remove(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &BlackboardValue)> {
        self.entries.iter()
    }
}
