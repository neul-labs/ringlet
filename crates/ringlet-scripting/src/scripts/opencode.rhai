// OpenCode configuration script
// Generates config files for OpenCode CLI agent

// OpenCode uses Anthropic-compatible API
// For self-auth, use native anthropic provider (OpenCode handles auth)
let config = #{
    model: ctx.profile.model,
    provider: if ctx.provider.type == "anthropic" || ctx.provider.type == "self" {
        "anthropic"
    } else {
        "anthropic-compatible"
    }
};

// Build environment variables
let env = #{};

// Only set API key env vars if not self-authenticating
if ctx.provider.type != "self" {
    env[ctx.provider.auth_env_key] = "${API_KEY}";

    // If using a custom endpoint, set it
    if ctx.provider.type != "anthropic" {
        config.api_base = ctx.profile.endpoint;
        env["ANTHROPIC_BASE_URL"] = ctx.profile.endpoint;
    }
}

// Return the output
#{
    files: #{
        ".opencode/config.json": json::encode_pretty(config)
    },
    env: env,
    hooks: (),
    mcp_servers: ()
}
