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

    /// Returns the `(x, y, z)` tuple if this is a [`BlackboardValue::Vec3`], otherwise `None`.
    pub fn as_vec3(&self) -> Option<(f32, f32, f32)> {
        match self {
            Self::Vec3 { x, y, z } => Some((*x, *y, *z)),
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

    /// Typed shorthand for `set(key, BlackboardValue::Bool(value))`.
    pub fn set_bool(&mut self, key: impl Into<String>, value: bool) {
        self.set(key, BlackboardValue::Bool(value));
    }

    /// Typed shorthand for `set(key, BlackboardValue::Int(value))`.
    pub fn set_int(&mut self, key: impl Into<String>, value: i64) {
        self.set(key, BlackboardValue::Int(value));
    }

    /// Typed shorthand for `set(key, BlackboardValue::Float(value))`.
    pub fn set_float(&mut self, key: impl Into<String>, value: f32) {
        self.set(key, BlackboardValue::Float(value));
    }

    /// Typed shorthand for `set(key, BlackboardValue::String(value.into()))`.
    pub fn set_string(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.set(key, BlackboardValue::String(value.into()));
    }

    /// Typed shorthand for `set(key, BlackboardValue::Vec3 { x, y, z })`.
    pub fn set_vec3(&mut self, key: impl Into<String>, x: f32, y: f32, z: f32) {
        self.set(key, BlackboardValue::Vec3 { x, y, z });
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

    /// Returns the `(x, y, z)` tuple stored under `key`, if it exists and is a [`BlackboardValue::Vec3`].
    ///
    /// # Example
    /// ```
    /// use fyrox_ai_free::Blackboard;
    /// let mut bb = Blackboard::new();
    /// bb.set_vec3("target", 1.0, 2.0, 3.0);
    /// assert_eq!(bb.get_vec3("target"), Some((1.0, 2.0, 3.0)));
    /// ```
    pub fn get_vec3(&self, key: &str) -> Option<(f32, f32, f32)> {
        self.get(key).and_then(|v| v.as_vec3())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_setters_and_vec3_accessor_roundtrip() {
        let mut bb = Blackboard::new();
        bb.set_bool("alive", true);
        bb.set_int("score", 42);
        bb.set_float("health", 0.75);
        bb.set_string("name", "Goblin");
        bb.set_vec3("pos", 1.0, 2.0, 3.0);

        assert_eq!(bb.get_bool("alive"), Some(true));
        assert_eq!(bb.get_int("score"), Some(42));
        assert_eq!(bb.get_float("health"), Some(0.75));
        assert_eq!(bb.get_string("name"), Some("Goblin"));
        assert_eq!(bb.get_vec3("pos"), Some((1.0, 2.0, 3.0)));

        // wrong-type / missing key returns None
        assert_eq!(bb.get_vec3("alive"), None);
        assert_eq!(bb.get_vec3("missing"), None);

        // BlackboardValue::as_vec3 directly
        assert_eq!(
            BlackboardValue::Vec3 { x: 4.0, y: 5.0, z: 6.0 }.as_vec3(),
            Some((4.0, 5.0, 6.0))
        );
        assert_eq!(BlackboardValue::Bool(true).as_vec3(), None);
    }
}
