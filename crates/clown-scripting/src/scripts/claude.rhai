// Claude Code configuration script
// Generates .claude/settings.json and other required files

// Determine if we're using a proxy
let using_proxy = ctx.profile.proxy_url != ();

// Build settings.json content
// When using proxy, always use anthropic-compatible with proxy URL
// For self-auth or native anthropic (without proxy), use "anthropic" type so Claude Code handles auth
let settings = #{
    model: ctx.profile.model,
    customApiKeyResponsibleParty: "user",
    hasCompletedOnboarding: true,
    hasAcknowledgedCostThreshold: true,
    primaryProvider: #{
        type: if using_proxy {
            // Proxy routes to multiple backends, use compatible mode
            "anthropic-compatible"
        } else if ctx.provider.type == "anthropic" || ctx.provider.type == "self" {
            "anthropic"
        } else {
            "anthropic-compatible"
        },
        baseUrl: if using_proxy {
            ctx.profile.proxy_url
        } else if ctx.provider.type == "anthropic" || ctx.provider.type == "self" {
            ()
        } else {
            ctx.profile.endpoint
        }
    }
};

// Add hooks configuration if present
// hooks_config is the full Claude Code hooks format (PreToolUse, PostToolUse, etc.)
if ctx.profile.hooks_config != () {
    settings.hooks = ctx.profile.hooks_config;
}

// Legacy: Add simple hooks if enabled (for backwards compatibility)
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
    // Set the provider's auth key for API key retrieval
    env[ctx.provider.auth_env_key] = "${API_KEY}";

    if using_proxy {
        // When using proxy, set ANTHROPIC_AUTH_TOKEN and proxy URL
        // The proxy will handle routing to the actual provider
        env["ANTHROPIC_AUTH_TOKEN"] = "${API_KEY}";
        env["ANTHROPIC_BASE_URL"] = ctx.profile.proxy_url;
    } else if ctx.provider.type == "anthropic" {
        // For native Anthropic, use ANTHROPIC_API_KEY
        env["ANTHROPIC_API_KEY"] = "${API_KEY}";
    } else {
        // For anthropic-compatible providers (like MiniMax),
        // Claude Code expects ANTHROPIC_AUTH_TOKEN and ANTHROPIC_BASE_URL
        env["ANTHROPIC_AUTH_TOKEN"] = "${API_KEY}";
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
