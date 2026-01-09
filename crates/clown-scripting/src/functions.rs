//! Built-in functions for Rhai scripts.

use rhai::{Dynamic, Engine, EvalAltResult, Map, Position};

/// Register all built-in functions with the engine.
pub fn register_all(engine: &mut Engine) {
    // JSON module
    engine.register_fn("json_encode", json_encode);
    engine.register_fn("json_encode_pretty", json_encode_pretty);

    // TOML module
    engine.register_fn("toml_encode", toml_encode);

    // Register modules for nicer syntax
    let mut json_module = rhai::Module::new();
    json_module.set_native_fn("encode", json_encode);
    json_module.set_native_fn("encode_pretty", json_encode_pretty);
    engine.register_static_module("json", json_module.into());

    let mut toml_module = rhai::Module::new();
    toml_module.set_native_fn("encode", toml_encode);
    engine.register_static_module("toml", toml_module.into());

    // String utilities
    engine.register_fn("indent", indent_string);
    engine.register_fn("trim_lines", trim_lines);
}

/// Encode a value as JSON.
fn json_encode(value: Dynamic) -> Result<String, Box<EvalAltResult>> {
    let json_value = dynamic_to_json(&value)?;
    serde_json::to_string(&json_value).map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("JSON encode failed: {}", e).into(),
            Position::NONE,
        ))
    })
}

/// Encode a value as pretty-printed JSON.
fn json_encode_pretty(value: Dynamic) -> Result<String, Box<EvalAltResult>> {
    let json_value = dynamic_to_json(&value)?;
    serde_json::to_string_pretty(&json_value).map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("JSON encode failed: {}", e).into(),
            Position::NONE,
        ))
    })
}

/// Encode a value as TOML.
fn toml_encode(value: Dynamic) -> Result<String, Box<EvalAltResult>> {
    let json_value = dynamic_to_json(&value)?;
    let toml_value: toml::Value = serde_json::from_value(json_value).map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("TOML encode failed: {}", e).into(),
            Position::NONE,
        ))
    })?;
    toml::to_string_pretty(&toml_value).map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("TOML encode failed: {}", e).into(),
            Position::NONE,
        ))
    })
}

/// Indent each line of a string.
fn indent_string(s: String, spaces: i64) -> String {
    let prefix = " ".repeat(spaces as usize);
    s.lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("{}{}", prefix, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Trim leading/trailing whitespace from each line.
fn trim_lines(s: String) -> String {
    s.lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert Rhai Dynamic to serde_json::Value.
fn dynamic_to_json(value: &Dynamic) -> Result<serde_json::Value, Box<EvalAltResult>> {
    if value.is::<()>() {
        Ok(serde_json::Value::Null)
    } else if value.is::<bool>() {
        Ok(serde_json::Value::Bool(value.clone().cast::<bool>()))
    } else if value.is::<i64>() {
        Ok(serde_json::Value::Number(value.clone().cast::<i64>().into()))
    } else if value.is::<f64>() {
        let f = value.clone().cast::<f64>();
        Ok(serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null))
    } else if value.is::<String>() {
        Ok(serde_json::Value::String(value.clone().cast::<String>()))
    } else if value.is::<rhai::Array>() {
        let arr = value.clone().cast::<rhai::Array>();
        let mut json_arr = Vec::with_capacity(arr.len());
        for item in arr {
            json_arr.push(dynamic_to_json(&item)?);
        }
        Ok(serde_json::Value::Array(json_arr))
    } else if value.is::<Map>() {
        let map = value.clone().cast::<Map>();
        let mut json_obj = serde_json::Map::new();
        for (k, v) in map {
            json_obj.insert(k.to_string(), dynamic_to_json(&v)?);
        }
        Ok(serde_json::Value::Object(json_obj))
    } else {
        // Try to convert to string as fallback
        Ok(serde_json::Value::String(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_encode() {
        let mut map = Map::new();
        map.insert("key".into(), "value".into());
        let result = json_encode(map.into()).unwrap();
        assert!(result.contains("\"key\""));
        assert!(result.contains("\"value\""));
    }

    #[test]
    fn test_indent() {
        let result = indent_string("line1\nline2".to_string(), 2);
        assert_eq!(result, "  line1\n  line2");
    }
}
