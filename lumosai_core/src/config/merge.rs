//! Configuration merge strategies and utilities

use serde_json::Value;

/// Merge two configuration values with different strategies
pub fn merge_configs(base: Value, update: Value, strategy: &str) -> Value {
    match strategy {
        "first_wins" => {
            if !base.is_null() {
                base
            } else {
                update
            }
        }
        "last_wins" => {
            if !update.is_null() {
                update
            } else {
                base
            }
        }
        "deep_merge" => deep_merge(base, update),
        _ => update,
    }
}

/// Deep merge configuration values
fn deep_merge(base: Value, update: Value) -> Value {
    match (base, update) {
        (Value::Object(mut base_map), Value::Object(update_map)) => {
            for (key, value) in update_map {
                if let Some(base_value) = base_map.get(&key) {
                    base_map.insert(key, deep_merge(base_value.clone(), value));
                } else {
                    base_map.insert(key, value);
                }
            }
            Value::Object(base_map)
        }
        (Value::Array(mut base_array), Value::Array(update_array)) => {
            base_array.extend(update_array);
            Value::Array(base_array)
        }
        (_, update) => update,
    }
}