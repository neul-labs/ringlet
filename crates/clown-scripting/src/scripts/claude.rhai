// Claude Code configuration script
// Generates .claude/settings.json and other required files

// Build settings.json content
let settings = #{
    model: ctx.profile.model,
    customApiKeyResponsibleParty: "user",
    hasCompletedOnboarding: true,
    hasAcknowledgedCostThreshold: true,
    primaryProvider: #{
        type: if ctx.provider.type == "anthropic" { "anthropic" } else { "anthropic-compatible" },
        baseUrl: if ctx.provider.type == "anthropic" { () } else { ctx.profile.endpoint }
    }
};

// Add hooks if enabled
let hooks = ();
if ctx.profile.hooks.len() > 0 {
    hooks = #{};
    for hook in ctx.profile.hooks {
        hooks[hook] = true;
    }
}

// Add MCP servers if enabled
let mcp_servers = ();
if ctx.profile.mcp_servers.len() > 0 {
    mcp_servers = [];
    for server in ctx.profile.mcp_servers {
        mcp_servers.push(#{
            name: server,
            enabled: true
        });
    }
}

// Build environment variables
let env = #{};

// Only set API key env var if not self-authenticating
if ctx.provider.type != "self" {
    env[ctx.provider.auth_env_key] = "${API_KEY}";  // Placeholder, replaced at runtime
    if ctx.provider.type != "anthropic" {
        env["ANTHROPIC_BASE_URL"] = ctx.profile.endpoint;
    }
}

// Return the output
#{
    files: #{
        ".claude/settings.json": json::encode_pretty(settings)
    },
    env: env,
    hooks: hooks,
    mcp_servers: mcp_servers
}
