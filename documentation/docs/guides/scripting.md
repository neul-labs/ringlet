# Scripting with Rhai

Clown uses [Rhai](https://rhai.rs/), an embedded scripting language for Rust, to generate agent-specific configuration files. Each agent has a `.rhai` script that receives context about the provider, profile, and user preferences, then outputs the required configuration.

---

## Why Rhai?

- **Extensibility**: Add new agents without recompiling ringlet
- **Customization**: Users can override scripts for special cases
- **Declarative**: Configuration logic is visible and editable
- **Sandboxed**: Scripts can't access filesystem or network directly
- **Future-proof**: New agent features can be supported by updating scripts

---

## Script Resolution Order

Scripts are resolved in this order:

1. `~/.config/ringlet/scripts/<agent-id>.rhai` (user override)
2. `registry/scripts/<agent-id>.rhai` (from GitHub registry)
3. Built-in scripts (compiled into binary)

---

## Script Interface

### Input Variables

Scripts receive these variables from ringlet:

```rhai
// === Provider Context ===
provider.id          // "minimax"
provider.name        // "MiniMax"
provider.type        // "anthropic-compatible"
provider.endpoint_id // "international" or "china"
provider.endpoint    // "https://api.minimax.io/anthropic"
provider.api_key     // API key (from keychain)
provider.model       // "MiniMax-M2.1"

// === Profile Context ===
profile.alias        // "work-minimax"
profile.home         // "/home/user/.claude-profiles/work-minimax"
profile.project_dir  // Current project directory (if applicable)

// === Agent Context ===
agent.id             // "claude"
agent.binary         // "claude"

// === User Preferences ===
prefs.hooks.auto_format       // true/false
prefs.hooks.auto_lint         // true/false
prefs.hooks.custom            // Map of custom hooks
prefs.mcp_servers.filesystem  // true/false
prefs.mcp_servers.github      // true/false
prefs.mcp_servers.custom      // Map of custom MCP servers
prefs.custom                  // Any custom key-value pairs
```

### Output Structure

Scripts must return a map with these keys:

```rhai
#{
    // Required: Files to generate (relative paths from profile.home)
    "files": #{
        ".claude/settings.json": json_content,
        ".claude.json": mcp_config_content
    },

    // Optional: Environment variables to inject at runtime
    "env": #{
        "ANTHROPIC_BASE_URL": "https://...",
        "ANTHROPIC_AUTH_TOKEN": "..."
    },

    // Optional: Hooks configuration (for agents that support them)
    "hooks": #{
        "PreToolUse": [...],
        "PostToolUse": [...],
        "Notification": [...],
        "Stop": [...]
    },

    // Optional: MCP servers (for agents that support them)
    "mcp_servers": #{
        "filesystem": #{
            "command": "npx",
            "args": ["-y", "@anthropic/mcp-filesystem"],
            "env": #{}
        }
    }
}
```

---

## Built-in Functions

Clown exposes these functions to Rhai scripts:

```rhai
// Encode a map as pretty-printed JSON
json::encode(map)  // Returns String

// Encode a map as TOML
toml::encode(map)  // Returns String
```

---

## Example Scripts

### Basic: Claude Code

```rhai
// Claude Code configuration script
// Generates ~/.claude/settings.json

let settings = #{
    "env": #{
        "ANTHROPIC_BASE_URL": provider.endpoint,
        "ANTHROPIC_AUTH_TOKEN": provider.api_key,
        "API_TIMEOUT_MS": "3000000",
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
        "ANTHROPIC_MODEL": provider.model,
        "ANTHROPIC_SMALL_FAST_MODEL": provider.model,
        "ANTHROPIC_DEFAULT_SONNET_MODEL": provider.model,
        "ANTHROPIC_DEFAULT_OPUS_MODEL": provider.model,
        "ANTHROPIC_DEFAULT_HAIKU_MODEL": provider.model
    }
};

#{
    "files": #{
        ".claude/settings.json": json::encode(settings)
    },
    "env": #{}
}
```

### With Hooks

```rhai
// Claude Code with hooks support

let env_config = #{
    "ANTHROPIC_BASE_URL": provider.endpoint,
    "ANTHROPIC_AUTH_TOKEN": provider.api_key,
    "API_TIMEOUT_MS": "3000000",
    "ANTHROPIC_MODEL": provider.model
};

// Build hooks based on user preferences
let hooks_config = #{};

if prefs.hooks.auto_format {
    hooks_config.PostToolUse = [
        #{
            "matcher": "Edit|Write|MultiEdit",
            "hooks": [
                #{
                    "type": "command",
                    "command": "prettier --write \"$CLAUDE_FILE_PATHS\" 2>/dev/null || true"
                }
            ]
        }
    ];
}

if prefs.hooks.auto_lint {
    let lint_hook = #{
        "matcher": "Edit|Write",
        "hooks": [#{ "type": "command", "command": "eslint --fix \"$CLAUDE_FILE_PATHS\"" }]
    };
    if hooks_config.PostToolUse == () {
        hooks_config.PostToolUse = [];
    }
    hooks_config.PostToolUse.push(lint_hook);
}

// Build settings.json
let settings = #{ "env": env_config };
if hooks_config.keys().len() > 0 {
    settings.hooks = hooks_config;
}

#{
    "files": #{
        ".claude/settings.json": json::encode(settings)
    },
    "env": #{},
    "hooks": hooks_config
}
```

### With MCP Servers

```rhai
// Claude Code with MCP server support

let env_config = #{
    "ANTHROPIC_BASE_URL": provider.endpoint,
    "ANTHROPIC_AUTH_TOKEN": provider.api_key,
    "ANTHROPIC_MODEL": provider.model
};

// Build MCP servers based on user preferences
let mcp_config = #{};

if prefs.mcp_servers.filesystem {
    mcp_config.filesystem = #{
        "command": "npx",
        "args": ["-y", "@anthropic/mcp-filesystem", profile.project_dir],
        "env": #{}
    };
}

if prefs.mcp_servers.github && prefs.mcp_servers.github_token != () {
    mcp_config.github = #{
        "command": "npx",
        "args": ["-y", "@anthropic/mcp-github"],
        "env": #{ "GITHUB_TOKEN": prefs.mcp_servers.github_token }
    };
}

// Add any custom MCP servers
if prefs.mcp_servers.custom != () {
    for name in prefs.mcp_servers.custom.keys() {
        mcp_config[name] = prefs.mcp_servers.custom[name];
    }
}

// Build output files
let files = #{
    ".claude/settings.json": json::encode(#{ "env": env_config })
};

if mcp_config.keys().len() > 0 {
    files[".claude.json"] = json::encode(#{ "mcpServers": mcp_config });
}

#{
    "files": files,
    "env": #{},
    "mcp_servers": mcp_config
}
```

### TOML Output: Codex CLI

```rhai
// Codex CLI configuration script
// Generates ~/.codex/config.toml

let provider_section = #{
    "name": provider.name + " Chat Completions API",
    "base_url": provider.endpoint,
    "env_key": "CLOWN_API_KEY",
    "wire_api": "chat",
    "requires_openai_auth": false
};

let profile_section = #{
    "model": "codex-" + provider.model,
    "model_provider": provider.id
};

let config = #{
    "model_providers": #{},
    "profiles": #{}
};
config.model_providers[provider.id] = provider_section;
config.profiles[profile.alias] = profile_section;

#{
    "files": #{
        ".codex/config.toml": toml::encode(config)
    },
    "env": #{
        "CLOWN_API_KEY": provider.api_key
    }
}
```

### Environment Only: Grok CLI

```rhai
// Grok CLI - pure environment variables, no config files

#{
    "files": #{},
    "env": #{
        "GROK_BASE_URL": provider.endpoint,
        "GROK_API_KEY": provider.api_key
    }
}
```

---

## User Preferences

Users can configure default preferences in `~/.config/ringlet/config.toml`:

```toml
[defaults]
provider = "anthropic"

[hooks]
auto_format = true
auto_lint = false

[hooks.custom.PostToolUse]
[[hooks.custom.PostToolUse]]
matcher = "Write"
type = "command"
command = "echo 'File written'"

[mcp_servers]
filesystem = true
github = false
github_token = ""

[mcp_servers.custom.my-server]
command = "node"
args = ["./my-mcp.js"]
```

---

## CLI Flags

Override preferences per profile:

```bash
# Enable specific hooks
ringlet profiles create claude work --provider minimax --hooks auto_format,auto_lint

# Enable MCP servers
ringlet profiles create claude work --provider minimax --mcp filesystem,github

# Minimal profile (no hooks, no MCP)
ringlet profiles create claude minimal --provider anthropic --bare
```

---

## Creating Custom Scripts

### 1. Create Script Directory

```bash
mkdir -p ~/.config/ringlet/scripts
```

### 2. Write Your Script

```rhai
// ~/.config/ringlet/scripts/claude.rhai
// Custom Claude Code configuration

let settings = #{
    "env": #{
        "ANTHROPIC_BASE_URL": provider.endpoint,
        "ANTHROPIC_AUTH_TOKEN": provider.api_key,
        "MY_CUSTOM_VAR": "custom-value"
    }
};

#{
    "files": #{
        ".claude/settings.json": json::encode(settings)
    },
    "env": #{}
}
```

### 3. Create a Profile

Your script will be used automatically:

```bash
ringlet profiles create claude custom --provider minimax
```

---

## Debugging Scripts

Use `ringlet scripts test` to validate a script:

```bash
# Test script with mock data
ringlet scripts test claude.rhai --provider minimax --alias test

# Show generated output without creating profile
ringlet profiles create claude test --provider minimax --dry-run
```

---

## Rhai Language Reference

Rhai uses syntax similar to JavaScript and Rust:

```rhai
// Variables
let x = 42;
let name = "Claude";

// Maps (object literals)
let config = #{
    "key": "value",
    "nested": #{
        "inner": 123
    }
};

// Arrays
let items = ["a", "b", "c"];

// Conditionals
if condition {
    // ...
} else {
    // ...
}

// Loops
for item in items {
    print(item);
}

// String interpolation
let msg = `Hello ${name}`;

// Map operations
config.new_key = "new value";
config["another"] = 456;
let keys = config.keys();
let len = config.keys().len();

// Null check
if value == () {
    // value is null/undefined
}
```

For full Rhai documentation, see [The Rhai Book](https://rhai.rs/book/).
