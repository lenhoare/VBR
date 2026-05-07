use serde_json::{json, Value};

pub struct Json;

impl Json {

    /// Parse a JSON string into a Value
    /// VBA equivalent: parsing with MSXML2 or custom parser
    pub fn parse(text: &str) -> Result<Value, String> {
        serde_json::from_str(text)
            .map_err(|e| e.to_string())
    }

    /// Create an empty JSON object
    /// VBA equivalent: CreateObject("Scripting.Dictionary")
    pub fn object() -> Value {
        json!({})
    }

    /// Create an empty JSON array
    pub fn array() -> Value {
        json!([])
    }

    /// Serialise a Value to a JSON string
    pub fn to_string(value: &Value) -> Result<String, String> {
        serde_json::to_string(value)
            .map_err(|e| e.to_string())
    }

    /// Serialise a Value to a pretty printed JSON string
    pub fn to_pretty(value: &Value) -> Result<String, String> {
        serde_json::to_string_pretty(value)
            .map_err(|e| e.to_string())
    }

    /// Check if a key exists in a JSON object
    pub fn has_key(value: &Value, key: &str) -> bool {
        value.get(key).is_some()
    }

    /// Get a string value from a JSON object
    pub fn get_string(value: &Value, key: &str) -> Result<String, String> {
        value.get(key)
            .ok_or_else(|| format!("Key '{}' not found", key))?
            .as_str()
            .ok_or_else(|| format!("Key '{}' is not a string", key))
            .map(|s| s.to_string())
    }

    /// Get an integer value from a JSON object
    pub fn get_int(value: &Value, key: &str) -> Result<i64, String> {
        value.get(key)
            .ok_or_else(|| format!("Key '{}' not found", key))?
            .as_i64()
            .ok_or_else(|| format!("Key '{}' is not an integer", key))
    }

    /// Get a float value from a JSON object
    pub fn get_float(value: &Value, key: &str) -> Result<f64, String> {
        value.get(key)
            .ok_or_else(|| format!("Key '{}' not found", key))?
            .as_f64()
            .ok_or_else(|| format!("Key '{}' is not a float", key))
    }

    /// Get a boolean value from a JSON object
    pub fn get_bool(value: &Value, key: &str) -> Result<bool, String> {
        value.get(key)
            .ok_or_else(|| format!("Key '{}' not found", key))?
            .as_bool()
            .ok_or_else(|| format!("Key '{}' is not a boolean", key))
    }

    /// Get an array from a JSON object
    pub fn get_array(value: &Value, key: &str) -> Result<Vec<Value>, String> {
        value.get(key)
            .ok_or_else(|| format!("Key '{}' not found", key))?
            .as_array()
            .ok_or_else(|| format!("Key '{}' is not an array", key))
            .map(|a| a.clone())
    }

    /// Set a value in a JSON object
    pub fn set(value: &mut Value, key: &str, val: Value) {
        value[key] = val;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_get() {
        let data = Json::parse(r#"{"name":"Alice","age":42}"#).unwrap();
        assert_eq!(Json::get_string(&data, "name").unwrap(), "Alice");
        assert_eq!(Json::get_int(&data, "age").unwrap(), 42);
    }

    #[test]
    fn test_object_and_serialise() {
        let mut obj = Json::object();
        Json::set(&mut obj, "name", serde_json::json!("Bob"));
        let text = Json::to_string(&obj).unwrap();
        assert!(text.contains("Bob"));
    }

    #[test]
    fn test_has_key() {
        let data = Json::parse(r#"{"name":"Alice"}"#).unwrap();
        assert!(Json::has_key(&data, "name"));
        assert!(!Json::has_key(&data, "age"));
    }

    #[test]
    fn test_get_array() {
        let data = Json::parse(r#"{"items":[1,2,3]}"#).unwrap();
        let arr = Json::get_array(&data, "items").unwrap();
        assert_eq!(arr.len(), 3);
    }
}
