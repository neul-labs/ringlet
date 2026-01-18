//! Rhai engine setup and execution.

use crate::functions;
use anyhow::{anyhow, Result};
use rhai::{Dynamic, Engine, Map, Scope, AST};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::debug;

/// Script execution context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptContext {
    /// Profile information.
    pub profile: ProfileContext,
    /// Provider information.
    pub provider: ProviderContext,
    /// Agent information.
    pub agent: AgentContext,
    /// User preferences.
    pub prefs: PrefsContext,
}

/// Profile context for scripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileContext {
    pub alias: String,
    pub home: PathBuf,
    pub model: String,
    pub endpoint: String,
    pub hooks: Vec<String>,
    pub mcp_servers: Vec<String>,
    /// Full hooks configuration as JSON (for Claude Code hooks).
    pub hooks_config: Option<serde_json::Value>,
    /// Proxy URL if proxy is enabled for this profile.
    pub proxy_url: Option<String>,
}

/// Provider context for scripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderContext {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub auth_env_key: String,
}

/// Agent context for scripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub id: String,
    pub name: String,
    pub binary: String,
}

/// User preferences context for scripts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrefsContext {
    /// Custom preferences map.
    #[serde(flatten)]
    pub custom: HashMap<String, String>,
}

/// Script execution output.
#[derive(Debug, Clone, Default)]
pub struct ScriptOutput {
    /// Files to write (relative path -> content).
    pub files: HashMap<String, String>,
    /// Environment variables to set.
    pub env: HashMap<String, String>,
    /// Additional command-line arguments to pass to the agent.
    pub args: Vec<String>,
    /// Optional hooks configuration.
    pub hooks: Option<serde_json::Value>,
    /// Optional MCP servers configuration.
    pub mcp_servers: Option<serde_json::Value>,
}

/// Rhai script engine.
pub struct ScriptEngine {
    engine: Engine,
}

impl ScriptEngine {
    /// Create a new script engine with sandboxed settings.
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Limit execution resources
        engine.set_max_operations(100_000);
        engine.set_max_string_size(1024 * 1024); // 1MB max string
        engine.set_max_array_size(10_000);
        engine.set_max_map_size(10_000);
        engine.set_max_call_levels(64);

        // Register custom functions
        functions::register_all(&mut engine);

        Self { engine }
    }

    /// Compile a script for faster execution.
    pub fn compile(&self, script: &str) -> Result<AST> {
        self.engine
            .compile(script)
            .map_err(|e| anyhow!("Failed to compile script: {}", e))
    }

    /// Run a script with the given context.
    pub fn run(&self, script: &str, context: &ScriptContext) -> Result<ScriptOutput> {
        let ast = self.compile(script)?;
        self.run_ast(&ast, context)
    }

    /// Run a compiled script with the given context.
    pub fn run_ast(&self, ast: &AST, context: &ScriptContext) -> Result<ScriptOutput> {
        let mut scope = Scope::new();

        // Convert context to Rhai dynamic values
        let context_dynamic = context_to_dynamic(context)?;
        scope.push_dynamic("ctx", context_dynamic);

        debug!("Running script with context: {:?}", context);

        // Execute script
        let result: Dynamic = self
            .engine
            .eval_ast_with_scope(&mut scope, ast)
            .map_err(|e| anyhow!("Script execution failed: {}", e))?;

        // Convert result to ScriptOutput
        dynamic_to_output(result)
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert ScriptContext to Rhai Dynamic.
fn context_to_dynamic(context: &ScriptContext) -> Result<Dynamic> {
    let mut map = Map::new();

    // Profile
    let mut profile = Map::new();
    profile.insert("alias".into(), context.profile.alias.clone().into());
    profile.insert(
        "home".into(),
        context.profile.home.to_string_lossy().to_string().into(),
    );
    profile.insert("model".into(), context.profile.model.clone().into());
    profile.insert("endpoint".into(), context.profile.endpoint.clone().into());
    profile.insert(
        "hooks".into(),
        context
            .profile
            .hooks
            .iter()
            .map(|s| Dynamic::from(s.clone()))
            .collect::<Vec<_>>()
            .into(),
    );
    profile.insert(
        "mcp_servers".into(),
        context
            .profile
            .mcp_servers
            .iter()
            .map(|s| Dynamic::from(s.clone()))
            .collect::<Vec<_>>()
            .into(),
    );
    // Add hooks_config as a dynamic value (JSON -> Rhai map)
    if let Some(ref hooks_json) = context.profile.hooks_config {
        let hooks_dynamic = json_to_dynamic(hooks_json.clone())?;
        profile.insert("hooks_config".into(), hooks_dynamic);
    } else {
        profile.insert("hooks_config".into(), Dynamic::UNIT);
    }
    // Add proxy_url if present
    if let Some(ref proxy_url) = context.profile.proxy_url {
        profile.insert("proxy_url".into(), proxy_url.clone().into());
    } else {
        profile.insert("proxy_url".into(), Dynamic::UNIT);
    }
    map.insert("profile".into(), profile.into());

    // Provider
    let mut provider = Map::new();
    provider.insert("id".into(), context.provider.id.clone().into());
    provider.insert("name".into(), context.provider.name.clone().into());
    provider.insert("type".into(), context.provider.provider_type.clone().into());
    provider.insert(
        "auth_env_key".into(),
        context.provider.auth_env_key.clone().into(),
    );
    map.insert("provider".into(), provider.into());

    // Agent
    let mut agent = Map::new();
    agent.insert("id".into(), context.agent.id.clone().into());
    agent.insert("name".into(), context.agent.name.clone().into());
    agent.insert("binary".into(), context.agent.binary.clone().into());
    map.insert("agent".into(), agent.into());

    // Prefs
    let mut prefs = Map::new();
    for (k, v) in &context.prefs.custom {
        prefs.insert(k.clone().into(), v.clone().into());
    }
    map.insert("prefs".into(), prefs.into());

    Ok(map.into())
}

/// Convert Rhai Dynamic result to ScriptOutput.
fn dynamic_to_output(result: Dynamic) -> Result<ScriptOutput> {
    let mut output = ScriptOutput::default();

    let map = result
        .try_cast::<Map>()
        .ok_or_else(|| anyhow!("Script must return an object"))?;

    // Extract files
    if let Some(files_dynamic) = map.get("files") {
        if let Some(files_map) = files_dynamic.clone().try_cast::<Map>() {
            for (key, value) in files_map {
                if let Some(content) = value.clone().try_cast::<String>() {
                    output.files.insert(key.to_string(), content);
                }
            }
        }
    }

    // Extract env
    if let Some(env_dynamic) = map.get("env") {
        if let Some(env_map) = env_dynamic.clone().try_cast::<Map>() {
            for (key, value) in env_map {
                if let Some(val) = value.clone().try_cast::<String>() {
                    output.env.insert(key.to_string(), val);
                }
            }
        }
    }

    // Extract args
    if let Some(args_dynamic) = map.get("args") {
        if let Some(args_arr) = args_dynamic.clone().try_cast::<rhai::Array>() {
            for arg in args_arr {
                if let Some(arg_str) = arg.clone().try_cast::<String>() {
                    output.args.push(arg_str);
                }
            }
        }
    }

    // Extract hooks (as JSON)
    if let Some(hooks_dynamic) = map.get("hooks") {
        output.hooks = Some(dynamic_to_json(hooks_dynamic.clone())?);
    }

    // Extract mcp_servers (as JSON)
    if let Some(mcp_dynamic) = map.get("mcp_servers") {
        output.mcp_servers = Some(dynamic_to_json(mcp_dynamic.clone())?);
    }

    Ok(output)
}

/// Convert Rhai Dynamic to serde_json::Value.
fn dynamic_to_json(value: Dynamic) -> Result<serde_json::Value> {
    if value.is::<()>() {
        Ok(serde_json::Value::Null)
    } else if value.is::<bool>() {
        Ok(serde_json::Value::Bool(value.cast::<bool>()))
    } else if value.is::<i64>() {
        Ok(serde_json::Value::Number(value.cast::<i64>().into()))
    } else if value.is::<f64>() {
        let f = value.cast::<f64>();
        Ok(serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null))
    } else if value.is::<String>() {
        Ok(serde_json::Value::String(value.cast::<String>()))
    } else if value.is::<rhai::Array>() {
        let arr = value.cast::<rhai::Array>();
        let mut json_arr = Vec::with_capacity(arr.len());
        for item in arr {
            json_arr.push(dynamic_to_json(item)?);
        }
        Ok(serde_json::Value::Array(json_arr))
    } else if value.is::<Map>() {
        let map = value.cast::<Map>();
        let mut json_obj = serde_json::Map::new();
        for (k, v) in map {
            json_obj.insert(k.to_string(), dynamic_to_json(v)?);
        }
        Ok(serde_json::Value::Object(json_obj))
    } else {
        // Try to convert to string as fallback
        Ok(serde_json::Value::String(value.to_string()))
    }
}

/// Convert serde_json::Value to Rhai Dynamic.
fn json_to_dynamic(value: serde_json::Value) -> Result<Dynamic> {
    match value {
        serde_json::Value::Null => Ok(Dynamic::UNIT),
        serde_json::Value::Bool(b) => Ok(Dynamic::from(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Dynamic::from(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Dynamic::from(f))
            } else {
                Ok(Dynamic::UNIT)
            }
        }
        serde_json::Value::String(s) => Ok(Dynamic::from(s)),
        serde_json::Value::Array(arr) => {
            let mut rhai_arr = rhai::Array::new();
            for item in arr {
                rhai_arr.push(json_to_dynamic(item)?);
            }
            Ok(Dynamic::from(rhai_arr))
        }
        serde_json::Value::Object(obj) => {
            let mut map = Map::new();
            for (k, v) in obj {
                map.insert(k.into(), json_to_dynamic(v)?);
            }
            Ok(Dynamic::from(map))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_script() {
        let engine = ScriptEngine::new();

        let script = r#"
            #{
                files: #{
                    "test.txt": "Hello, " + ctx.profile.alias
                },
                env: #{
                    "TEST_VAR": "test_value"
                }
            }
        "#;

        let context = ScriptContext {
            profile: ProfileContext {
                alias: "myprofile".to_string(),
                home: PathBuf::from("/home/test"),
                model: "test-model".to_string(),
                endpoint: "https://api.test.com".to_string(),
                hooks: vec![],
                mcp_servers: vec![],
                hooks_config: None,
                proxy_url: None,
            },
            provider: ProviderContext {
                id: "test".to_string(),
                name: "Test Provider".to_string(),
                provider_type: "anthropic".to_string(),
                auth_env_key: "TEST_API_KEY".to_string(),
            },
            agent: AgentContext {
                id: "test".to_string(),
                name: "Test Agent".to_string(),
                binary: "test".to_string(),
            },
            prefs: PrefsContext::default(),
        };

        let output = engine.run(script, &context).unwrap();

        assert_eq!(output.files.get("test.txt"), Some(&"Hello, myprofile".to_string()));
        assert_eq!(output.env.get("TEST_VAR"), Some(&"test_value".to_string()));
    }

    #[test]
    fn test_json_encode() {
        let engine = ScriptEngine::new();

        let script = r#"
            let obj = #{ name: "test", value: 42 };
            #{
                files: #{
                    "config.json": json::encode(obj)
                },
                env: #{}
            }
        "#;

        let context = ScriptContext {
            profile: ProfileContext {
                alias: "test".to_string(),
                home: PathBuf::from("/home/test"),
                model: "test".to_string(),
                endpoint: "https://test.com".to_string(),
                hooks: vec![],
                mcp_servers: vec![],
                hooks_config: None,
                proxy_url: None,
            },
            provider: ProviderContext {
                id: "test".to_string(),
                name: "Test".to_string(),
                provider_type: "anthropic".to_string(),
                auth_env_key: "KEY".to_string(),
            },
            agent: AgentContext {
                id: "test".to_string(),
                name: "Test".to_string(),
                binary: "test".to_string(),
            },
            prefs: PrefsContext::default(),
        };

        let output = engine.run(script, &context).unwrap();
        let json_content = output.files.get("config.json").unwrap();
        assert!(json_content.contains("\"name\""));
        assert!(json_content.contains("\"test\""));
    }
}
